use crate::{
  aggregates::structs::CommunityAggregates,
  newtypes::CommunityId,
  schema::community_aggregates,
};
use diesel::{result::Error, *};

impl CommunityAggregates {
  pub fn read(conn: &mut PgConnection, community_id: CommunityId) -> Result<Self, Error> {
    community_aggregates::table
      .filter(community_aggregates::community_id.eq(community_id))
      .first::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    aggregates::community_aggregates::CommunityAggregates,
    source::{
      comment::{Comment, CommentInsertForm},
      community::{Community, CommunityFollower, CommunityFollowerForm, CommunityInsertForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
    },
    traits::{Crud, Followable},
    utils::establish_unpooled_connection,
  };
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_crud() {
    let conn = &mut establish_unpooled_connection();

    let inserted_instance = Instance::create(conn, "my_domain.tld").unwrap();

    let new_person = PersonInsertForm::builder()
      .name("thommy_community_agg".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(conn, &new_person).unwrap();

    let another_person = PersonInsertForm::builder()
      .name("jerry_community_agg".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let another_inserted_person = Person::create(conn, &another_person).unwrap();

    let new_community = CommunityInsertForm::builder()
      .name("TIL_community_agg".into())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(conn, &new_community).unwrap();

    let another_community = CommunityInsertForm::builder()
      .name("TIL_community_agg_2".into())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let another_inserted_community = Community::create(conn, &another_community).unwrap();

    let first_person_follow = CommunityFollowerForm {
      community_id: inserted_community.id,
      person_id: inserted_person.id,
      pending: false,
    };

    CommunityFollower::follow(conn, &first_person_follow).unwrap();

    let second_person_follow = CommunityFollowerForm {
      community_id: inserted_community.id,
      person_id: another_inserted_person.id,
      pending: false,
    };

    CommunityFollower::follow(conn, &second_person_follow).unwrap();

    let another_community_follow = CommunityFollowerForm {
      community_id: another_inserted_community.id,
      person_id: inserted_person.id,
      pending: false,
    };

    CommunityFollower::follow(conn, &another_community_follow).unwrap();

    let new_post = PostInsertForm::builder()
      .name("A test post".into())
      .creator_id(inserted_person.id)
      .community_id(inserted_community.id)
      .build();

    let inserted_post = Post::create(conn, &new_post).unwrap();

    let comment_form = CommentInsertForm::builder()
      .content("A test comment".into())
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .build();

    let inserted_comment = Comment::create(conn, &comment_form, None).unwrap();

    let child_comment_form = CommentInsertForm::builder()
      .content("A test comment".into())
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .build();

    let _inserted_child_comment =
      Comment::create(conn, &child_comment_form, Some(&inserted_comment.path)).unwrap();

    let community_aggregates_before_delete =
      CommunityAggregates::read(conn, inserted_community.id).unwrap();

    assert_eq!(2, community_aggregates_before_delete.subscribers);
    assert_eq!(1, community_aggregates_before_delete.posts);
    assert_eq!(2, community_aggregates_before_delete.comments);

    // Test the other community
    let another_community_aggs =
      CommunityAggregates::read(conn, another_inserted_community.id).unwrap();
    assert_eq!(1, another_community_aggs.subscribers);
    assert_eq!(0, another_community_aggs.posts);
    assert_eq!(0, another_community_aggs.comments);

    // Unfollow test
    CommunityFollower::unfollow(conn, &second_person_follow).unwrap();
    let after_unfollow = CommunityAggregates::read(conn, inserted_community.id).unwrap();
    assert_eq!(1, after_unfollow.subscribers);

    // Follow again just for the later tests
    CommunityFollower::follow(conn, &second_person_follow).unwrap();
    let after_follow_again = CommunityAggregates::read(conn, inserted_community.id).unwrap();
    assert_eq!(2, after_follow_again.subscribers);

    // Remove a parent comment (the comment count should also be 0)
    Post::delete(conn, inserted_post.id).unwrap();
    let after_parent_post_delete = CommunityAggregates::read(conn, inserted_community.id).unwrap();
    assert_eq!(0, after_parent_post_delete.comments);
    assert_eq!(0, after_parent_post_delete.posts);

    // Remove the 2nd person
    Person::delete(conn, another_inserted_person.id).unwrap();
    let after_person_delete = CommunityAggregates::read(conn, inserted_community.id).unwrap();
    assert_eq!(1, after_person_delete.subscribers);

    // This should delete all the associated rows, and fire triggers
    let person_num_deleted = Person::delete(conn, inserted_person.id).unwrap();
    assert_eq!(1, person_num_deleted);

    // Delete the community
    let community_num_deleted = Community::delete(conn, inserted_community.id).unwrap();
    assert_eq!(1, community_num_deleted);

    let another_community_num_deleted =
      Community::delete(conn, another_inserted_community.id).unwrap();
    assert_eq!(1, another_community_num_deleted);

    // Should be none found, since the creator was deleted
    let after_delete = CommunityAggregates::read(conn, inserted_community.id);
    assert!(after_delete.is_err());
  }
}
