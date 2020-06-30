use nvim_rs::create::tokio as create;

pub async fn start_nvim() {
    let (mut nvim, _handle, _child) = create::new_child(nvim_rs::rpc::handler::Dummy::new())
        .await
        .unwrap();
}
