pub mod async_network;
use tokio::signal;


#[tokio::main]
async fn main() {
    let ctrl_c_handler = tokio::spawn(async {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        println!("Ctrl+C received, shutting down...");
    });
    let server = tokio::spawn(async move{
        println!("Server is running...");
        async_network::run_server().await;
    });

    tokio::select! {
        _ = server => {},
        _ = ctrl_c_handler => {},
    }
    
    println!("Server shutdown gracefully");
    //서버 종료 작업
}