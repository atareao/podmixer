export interface Publisher {
    id: string;
    name: string;
    publisher_type: 'telegram' | 'x' | 'mastodon' | 'matrix';
    config: PublisherConfig;
    template: string;
    active: boolean;
    created_at: string;
    updated_at: string;
}

export interface PublisherConfig {
    bot_token?: string;
    chat_id?: string;
    message_thread_id?: string;
    client_id?: string;
    client_secret?: string;
    access_token?: string;
    refresh_token?: string;
    redirect_uri?: string;
    server_url?: string;
    access_token_mastodon?: string;
    homeserver_url?: string;
    room_id?: string;
    access_token_matrix?: string;
}

export interface PublishLog {
    id: string;
    publisher_id: string;
    publisher_name: string;
    publisher_type: string;
    episode_title: string;
    status: string;
    message: string;
    created_at: string;
}