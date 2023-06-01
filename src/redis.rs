use mockall::mock;
use redis::{aio::Connection, AsyncCommands, FromRedisValue, ToRedisArgs};

#[async_trait]
pub trait RedisConnection {
    async fn get_async<K: ToRedisArgs + Send + Sync, RV: FromRedisValue>(
        &mut self,
        key: K,
    ) -> Option<RV>;

    async fn set_async<K: ToRedisArgs + Send + Sync, V: ToRedisArgs + Send + Sync>(
        &mut self,
        key: K,
        value: V,
    ) -> ();
}

#[async_trait]
impl RedisConnection for Connection {
    async fn get_async<K: ToRedisArgs + Send + Sync, RV: FromRedisValue>(
        &mut self,
        key: K,
    ) -> Option<RV> {
        self.get::<K, RV>(key).await.ok()
    }

    async fn set_async<K: ToRedisArgs + Send + Sync, V: ToRedisArgs + Send + Sync>(
        &mut self,
        key: K,
        value: V,
    ) -> () {
        let _ = self.set::<K, V, ()>(key, value).await;
    }
}

mock! {
    pub RedisConnection {
        async fn get_async<'a, K: ToRedisArgs + Send + Sync + 'static, RV: FromRedisValue + 'static>(
            &'a mut self,
            key: K,
        ) -> Option<RV>;

        async fn set_async<
        'a,
        K: ToRedisArgs + Send + Sync + 'static,
        V: ToRedisArgs + Send + Sync + 'static,
        RV: FromRedisValue + 'static,
    >(
        &'a mut self,
        key: K,
        value: V,
    ) -> ();
    }
}
