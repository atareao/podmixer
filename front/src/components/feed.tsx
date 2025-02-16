import React from 'react';
import TextField from '@mui/material/TextField';
import Checkbox from '@mui/material/Checkbox';
import FormControlLabel from '@mui/material/FormControlLabel';
import Grid from '@mui/material/Grid2';
import Button from '@mui/material/Button';
import { BASE_URL } from '../constants';

interface FeedState {
    title: string
    link: string
    imageUrl: string
    category: string
    rating: string
    description: string
    author: string
    explicit: boolean
    keywords: string
}

export default class Feed extends React.Component<{}, FeedState> {
    constructor(props: {}) {
        super(props);
        this.state = {
            title: "",
            link: "",
            imageUrl: "",
            category: "",
            rating: "",
            description: "",
            author: "",
            explicit: false,
            keywords: ""
        }
    }

    onClick = async () => {
        console.log("Submitting feed");
        try {
            const body=`{
                "title": ${this.state.title ? JSON.stringify(this.state.title) : "\"\""},
                "link": ${this.state.link ? JSON.stringify(this.state.link) : "\"\""},
                "image_url": ${this.state.imageUrl ? JSON.stringify(this.state.imageUrl) : "\"\""},
                "category": ${this.state.category ? JSON.stringify(this.state.category) : "\"\""},
                "rating": ${this.state.rating ? JSON.stringify(this.state.rating) : "\"\""},
                "description": ${this.state.description ? JSON.stringify(this.state.description) : "\"\""},
                "author": ${this.state.author ? JSON.stringify(this.state.author) : "\"\""},
                "explicit": ${JSON.stringify(this.state.explicit)},
                "keywords": ${this.state.keywords ? JSON.stringify(this.state.keywords) : "\"\""}
                }`;
            console.log(`Submitting feed: ${body}`);
            const response = await fetch(`${BASE_URL}/api/v1/config/feed`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: body
            });
            const responseJson = await response.json();
            if (response.ok) {
                console.log("Response:", JSON.stringify(responseJson));
                const feed = responseJson.data;
                this.setState({
                    title: feed.title,
                    link: feed.link,
                    imageUrl: feed.image_url,
                    category: feed.category,
                    rating: feed.rating,
                    description: feed.description,
                    author: feed.author,
                    explicit: feed.explicit,
                    keywords: feed.keywords,
                });
           }
        } catch (error) {
            console.error('Error:', error);
        }
    }

    loadData = async () => {
        console.log("Loading data");
        try {
            const response = await fetch(`${BASE_URL}/api/v1/config/feed`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                },
            });
            const responseJson = await response.json();
            if (response.ok) {
                console.log("Response:", JSON.stringify(responseJson));
                const feed = responseJson.data;
                this.setState({
                    title: feed.title,
                    link: feed.link,
                    imageUrl: feed.image_url,
                    category: feed.category,
                    rating: feed.rating,
                    description: feed.description,
                    author: feed.author,
                    explicit: feed.explicit,
                    keywords: feed.keywords,
                });
            }
        } catch (error) {
            console.error('Error:', error);
        }
    }

    componentDidMount = () => {
        console.log("Mounting page");
        this.loadData();
    }

    render = () => {
        return (
            <Grid container spacing={1}>
                <Grid size={4}>
                    <TextField fullWidth label="Título" variant="outlined" value={this.state.title} onChange={(e) => this.setState({ title: e.target.value })} />
                </Grid>
                <Grid size={4}>
                    <TextField fullWidth label="Link" variant="outlined" value={this.state.link} onChange={(e) => this.setState({ link: e.target.value })} />
                </Grid>
                <Grid size={4}>
                    <TextField fullWidth label="Image Url" variant="outlined" value={this.state.imageUrl} onChange={(e) => this.setState({ imageUrl: e.target.value })} />
                </Grid>
                <Grid size={4}>
                    <TextField fullWidth label="Category" variant="outlined" value={this.state.category} onChange={(e) => this.setState({ category: e.target.value })} />
                </Grid>
                <Grid size={4}>
                    <TextField fullWidth label="Rating" variant="outlined" value={this.state.rating} onChange={(e) => this.setState({ rating: e.target.value })} />
                </Grid>
                <Grid size={4}>
                    <TextField fullWidth label="Description" variant="outlined" value={this.state.description} onChange={(e) => this.setState({ description: e.target.value })} />
                </Grid>
                <Grid size={4}>
                    <TextField fullWidth label="Author" variant="outlined" value={this.state.author} onChange={(e) => this.setState({ author: e.target.value })} />
                </Grid>
                <Grid size={4}>
                    <FormControlLabel control={<Checkbox checked={this.state.explicit} onChange={(e) => this.setState({ explicit: e.target.checked })} />} label="Explicit" />
                </Grid>
                <Grid size={4}>
                    <TextField fullWidth label="Keywords" variant="outlined" value={this.state.keywords} onChange={(e) => this.setState({ keywords: e.target.value })} />
                </Grid>
                <Grid size={16}>
                    <Button variant="contained" onClick={this.onClick}>Guardar</Button>
                </Grid>
            </Grid>
        );
    }
}

