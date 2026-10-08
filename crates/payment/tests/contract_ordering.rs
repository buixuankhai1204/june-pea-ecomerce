//! What payment expects from the ordering service, checked against ordering's real use cases.
//!
//! In the microservice version of this setup the ordering team would verify a Pact. Here both
//! services are crates in one binary, so the consumer side runs the other service's actual code
//! (with an in-memory repository, no database) through `OrderingAdapter` and checks each thing
//! `OrderLookup` promises. If ordering changes in a way payment can't live with, this fails.

use ordering::domain::model::{NewOrderItem, OrderStatus};
use ordering::domain::repository::OrderRepository;
use ordering::infrastructure::persistence::memory::InMemoryOrderRepository;
use ordering::usecase::{
    get_order::GetOrderUsecase, place_order::PlaceOrderUsecase,
    update_order_status::UpdateOrderStatusUsecase,
};
use payment::domain::{OrderLookup, OrderState};
use payment::infrastructure::ordering::OrderingAdapter;
use shared::error::AppError;
use shared::testing::{NoopExecutor, NoopUnitOfWork};
use std::sync::Arc;
use uuid::Uuid;

struct Ordering {
    adapter: OrderingAdapter,
    place: PlaceOrderUsecase,
    repo: Arc<InMemoryOrderRepository>,
}

fn ordering() -> Ordering {
    let repo = Arc::new(InMemoryOrderRepository::default());
    let uow = Arc::new(NoopUnitOfWork);
    Ordering {
        adapter: OrderingAdapter::new(
            Arc::new(GetOrderUsecase::new(repo.clone(), uow.clone())),
            Arc::new(UpdateOrderStatusUsecase::new(repo.clone(), uow.clone())),
        ),
        place: PlaceOrderUsecase::new(repo.clone(), uow),
        repo,
    }
}

async fn place(o: &Ordering, items: &[(i32, i64)]) -> Uuid {
    let items = items
        .iter()
        .map(|&(quantity, unit_price)| NewOrderItem {
            variant_id: Uuid::new_v4(),
            quantity,
            unit_price,
        })
        .collect();
    o.place.execute(None, items).await.unwrap()
}

#[tokio::test]
async fn a_placed_order_is_found_pending_with_its_total_in_vnd() {
    let o = ordering();
    let id = place(&o, &[(2, 50_000), (1, 25_000)]).await;

    let found = o.adapter.find(id).await.unwrap();

    assert_eq!(found.id, id);
    assert_eq!(found.total, 125_000);
    assert_eq!(found.state, OrderState::Pending);
}

#[tokio::test]
async fn an_unknown_order_is_not_found() {
    let o = ordering();

    let err = o.adapter.find(Uuid::new_v4()).await.unwrap_err();

    assert!(matches!(err, AppError::NotFound(_)));
}

#[tokio::test]
async fn marking_an_order_completed_shows_on_the_next_lookup() {
    let o = ordering();
    let id = place(&o, &[(1, 10_000)]).await;

    o.adapter.mark_completed(id).await.unwrap();

    assert_eq!(o.adapter.find(id).await.unwrap().state, OrderState::Completed);
}

#[tokio::test]
async fn every_ordering_status_maps_to_a_state() {
    let o = ordering();
    let id = place(&o, &[(1, 10_000)]).await;

    for (status, state) in [
        (OrderStatus::Pending, OrderState::Pending),
        (OrderStatus::Cancelled, OrderState::Cancelled),
        (OrderStatus::Completed, OrderState::Completed),
    ] {
        o.repo.update_order_status(&mut NoopExecutor, id, status).await.unwrap();
        assert_eq!(o.adapter.find(id).await.unwrap().state, state);
    }
}
