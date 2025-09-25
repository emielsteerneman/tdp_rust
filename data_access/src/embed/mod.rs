mod fastembed_client;

pub trait EmbedClient {
    fn embed_string(self, string: String) -> Vec<f32>;
}
