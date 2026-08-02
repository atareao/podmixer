export default interface Podcast {
    id?: number;
    name: string;
    url: string;
    last_pub_date: Date;
    active: boolean;
    created_at: Date;
    updated_at: Date;
}
