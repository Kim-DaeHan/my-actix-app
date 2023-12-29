use crate::error::MyError;
use crate::models::posts::{Post, PostData};
use crate::schema::posts::{self, dsl::*};
use crate::PgPool;
use actix_web::{http::header::ContentType, web, web::Data, HttpRequest, HttpResponse, Result};
use chrono::Utc;
use diesel::prelude::*;
use log::{info, warn};
use serde_json::to_vec;
use uuid::Uuid;

pub async fn get_posts(pool: Data<PgPool>) -> Result<HttpResponse, MyError> {
    info!("로깅 테스트");
    warn!("로깅 테스트2");
    let conn = &mut pool.get().expect("Couldn't get DB connection from pool");
    // use crate::schema::posts::{dsl::*}로 인해서 posts::table을 posts로 사용가능
    let post_list = posts.load::<Post>(conn).expect("error");

    println!("{:?}", post_list);
    if let Ok(post_data) =
        posts
            .select((body, title, posts::id, published))
            .load::<(String, String, String, bool)>(conn)
    {
        let json_bytes = to_vec(&post_data).expect("Failed to serialize posts to JSON");

        Ok(HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(json_bytes))
    } else {
        // 서버 에러
        Err(MyError::InternalError)
    }
}

pub async fn get_posts_by_id(
    req: HttpRequest,
    pool: Data<PgPool>,
) -> Result<HttpResponse, MyError> {
    if let Some(post_id) = req.match_info().get("id") {
        let conn = &mut pool.get().expect("Couldn't get DB connection from pool");

        if let Ok(post_data) = posts
            // .find(post_id)
            .filter(posts::id.eq(post_id))
            .select((body, title, posts::id, published))
            // get_result: 주어진 조건에 해당하는 하나의 결과를 반환, 결과가 여러 개거나 없으면 에러(정확히 하나의 결과가 예상되는 상황)
            .get_result::<(String, String, String, bool)>(conn)
        // first: 조건에 해당하는 모든 결과 중 첫 번째 결과 반환
        // .first::<(String, String, String, bool)>(conn)
        // load: 여러 레코드를 로드하고 벡터로 반환, 결과를 단일 값이 아닌 여러 레코드로 받아오려 할 때 사용
        // .load::<(String, String, String, bool)>(conn)
        {
            let json_bytes = to_vec(&post_data).expect("Failed to serialize posts to JSON");
            Ok(HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(json_bytes))
        } else {
            // id로 조회했는데 없을 경우
            // Err(MyError::InternalError)
            Ok(HttpResponse::Ok()
                .content_type(ContentType::json())
                .body("no result"))
        }
    } else {
        // id가 없는 경우에도 에러 처리
        Err(MyError::BadClientData)
    }
}

pub async fn create_posts(
    _body: web::Json<PostData>,
    pool: Data<PgPool>,
) -> Result<HttpResponse, MyError> {
    let post = PostData {
        id: Some(Uuid::new_v4().to_string()),
        .._body.into_inner()
    };

    let conn = &mut pool.get().expect("Couldn't get DB connection from pool");

    diesel::insert_into(posts)
        .values(post)
        .execute(conn)
        .expect("Error creating new post");

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body("created new post"))
}

pub async fn update_posts(
    _body: web::Json<PostData>,
    pool: Data<PgPool>,
) -> Result<HttpResponse, MyError> {
    let post_data = _body.into_inner();
    let updated_date = Some(Utc::now().naive_utc());
    if post_data.id.is_some() {
        let conn = &mut pool.get().expect("Couldn't get DB connection from pool");

        let post = PostData {
            id: None,
            updated_at: updated_date,
            ..post_data
        };

        if diesel::update(posts.find(post_data.id.unwrap()))
            .set(post)
            // .get_result::<Post>(conn)
            .execute(conn)
            .expect("Error updating post by id")
            == 0
        {
            Err(MyError::BadClientData)
        } else {
            Ok(HttpResponse::Ok()
                .content_type(ContentType::json())
                .body("updated new post"))
        }
    } else {
        //return이 있으면 update_posts(전체 함수)의 반환값, 없으면 해당 블록의 반환 값
        Err(MyError::BadClientData)
    }
}

pub async fn delete_posts_by_id(
    req: HttpRequest,
    pool: Data<PgPool>,
) -> Result<HttpResponse, MyError> {
    if let Some(post_id) = req.match_info().get("id") {
        let conn = &mut pool.get().expect("Couldn't get DB connection from pool");

        if diesel::delete(posts.find(post_id))
            .execute(conn)
            .expect("Error deleting post by id")
            == 0
        {
            Err(MyError::BadClientData)
        } else {
            Ok(HttpResponse::Ok()
                .content_type(ContentType::json())
                .body("deleted post"))
        }
    } else {
        // id가 없는 경우에도 에러 처리
        Err(MyError::BadClientData)
    }
}
