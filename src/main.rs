use elevator_simulator;
#[tokio::main]
async fn main() {
    elevator_simulator::run_simulation().await
}
