use redis::{aio::ConnectionManager, AsyncCommands, Client};

pub fn init_client(redis_url: &str) -> anyhow::Result<Client> {
    let client = Client::open(redis_url)?;
    Ok(client)
}

pub async fn get_connection(client: &Client) -> anyhow::Result<ConnectionManager> {
    let connection = client.get_connection_manager().await?;
    Ok(connection)
}

pub async fn cache_result(
    client: &Client,
    key: &str,
    value: &str,
    ttl_seconds: u64,
) -> anyhow::Result<()> {
    let mut conn = get_connection(client).await?;
    conn.set_ex::<_, _, ()>(key, value, ttl_seconds).await?;
    Ok(())
}

pub async fn get_cached_result(client: &Client, key: &str) -> anyhow::Result<Option<String>> {
    let mut conn = get_connection(client).await?;
    let cached: Option<String> = conn.get(key).await?;
    Ok(cached)
}
