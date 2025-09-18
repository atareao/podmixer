import React from 'react';
import TextField from '@mui/material/TextField';
import Checkbox from '@mui/material/Checkbox';
import FormControlLabel from '@mui/material/FormControlLabel';
import Grid from '@mui/material/Grid';
import Button from '@mui/material/Button';
import { BASE_URL } from '../constants';

interface FeedState {
    title: string;
    subtitle: string;
    summary: string;
    link: string;
    imageUrl: string;
    category: string;
    rating: string;
    description: string;
    author: string;
    explicit: boolean;
    keywords: string;
    owner_name: string;
    owner_email: string;

}

export default class Feed extends React.Component<{}, FeedState> {
    constructor(props: {}) {
        super(props);
        this.state = {
            title: "",
            subtitle: "",
            summary: "",
            link: "",
            imageUrl: "",
            category: "",
            rating: "",
            description: "",
            author: "",
            explicit: false,
            keywords: "",
            owner_name: "",
            owner_email: "",
        }
    }

    onClick = async () => {
        console.log("Submitting feed");
        try {
            const body=`{
                "title": ${this.state.title ? JSON.stringify(this.state.title) : "\"\""},
                "subtitle": ${this.state.subtitle ? JSON.stringify(this.state.subtitle) : "\"\""},
                "summary": ${this.state.summary ? JSON.stringify(this.state.summary) : "\"\""},
                "link": ${this.state.link ? JSON.stringify(this.state.link) : "\"\""},
                "image_url": ${this.state.imageUrl ? JSON.stringify(this.state.imageUrl) : "\"\""},
                "category": ${this.state.category ? JSON.stringify(this.state.category) : "\"\""},
                "rating": ${this.state.rating ? JSON.stringify(this.state.rating) : "\"\""},
                "description": ${this.state.description ? JSON.stringify(this.state.description) : "\"\""},
                "author": ${this.state.author ? JSON.stringify(this.state.author) : "\"\""},
                "explicit": ${JSON.stringify(this.state.explicit)},
                "keywords": ${this.state.keywords ? JSON.stringify(this.state.keywords) : "\"\""},
                "owner_name": ${this.state.owner_name ? JSON.stringify(this.state.owner_name) : "\"\""},
                "owner_email": ${this.state.owner_email ? JSON.stringify(this.state.owner_email) : "\"\""}
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
                    subtitle: feed.subtitle,
                    summary: feed.summary,
                    link: feed.link,
                    imageUrl: feed.image_url,
                    category: feed.category,
                    rating: feed.rating,
                    description: feed.description,
                    author: feed.author,
                    explicit: feed.explicit,
                    keywords: feed.keywords,
                    owner_name: feed.owner_name,
                    owner_email: feed.owner_email,

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
                    subtitle: feed.subtitle,
                    summary: feed.summary,
                    link: feed.link,
                    imageUrl: feed.image_url,
                    category: feed.category,
                    rating: feed.rating,
                    description: feed.description,
                    author: feed.author,
                    explicit: feed.explicit,
                    keywords: feed.keywords,
                    owner_name: feed.owner_name,
                    owner_email: feed.owner_email,
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
            <Grid
                container
                spacing={2}
                display='flex'
                flexDirection='row'
                justifyContent='center'
                alignContent='center'
            >
                <Grid size={{xs: 12, sm: 6, md: 4}}>
                    <TextField fullWidth label="Título" variant="outlined" value={this.state.title} onChange={(e) => this.setState({ title: e.target.value })} />
                </Grid>
                <Grid size={{xs: 12, sm: 6, md: 4}}>
                    <TextField fullWidth label="Subtítulo" variant="outlined" value={this.state.subtitle} onChange={(e) => this.setState({ subtitle: e.target.value })} />
                </Grid>
                <Grid size={{xs: 12, sm: 6, md: 4}}>
                    <TextField fullWidth label="Resumen" variant="outlined" value={this.state.summary} onChange={(e) => this.setState({ summary: e.target.value })} />
                </Grid>
                <Grid size={{xs: 12, sm: 6, md: 4}}>
                    <TextField fullWidth label="Link" variant="outlined" value={this.state.link} onChange={(e) => this.setState({ link: e.target.value })} />
                </Grid>
                <Grid size={{xs: 12, sm: 6, md: 4}}>
                    <TextField fullWidth label="Image Url" variant="outlined" value={this.state.imageUrl} onChange={(e) => this.setState({ imageUrl: e.target.value })} />
                </Grid>
                <Grid size={{xs: 12, sm: 6, md: 4}}>
                    <TextField fullWidth label="Category" variant="outlined" value={this.state.category} onChange={(e) => this.setState({ category: e.target.value })} />
                </Grid>
                <Grid size={{xs: 12, sm: 6, md: 4}}>
                    <TextField fullWidth label="Rating" variant="outlined" value={this.state.rating} onChange={(e) => this.setState({ rating: e.target.value })} />
                </Grid>
                <Grid size={{xs: 12, sm: 6, md: 4}}>
                    <TextField fullWidth label="Description" variant="outlined" value={this.state.description} onChange={(e) => this.setState({ description: e.target.value })} />
                </Grid>
                <Grid size={{xs: 12, sm: 6, md: 4}}>
                    <TextField fullWidth label="Author" variant="outlined" value={this.state.author} onChange={(e) => this.setState({ author: e.target.value })} />
                </Grid>
                <Grid size={{xs: 12, sm: 6, md: 4}}>
                    <TextField fullWidth label="Nombre del propietario" variant="outlined" value={this.state.owner_name} onChange={(e) => this.setState({ owner_name: e.target.value })} />
                </Grid>
                <Grid size={{xs: 12, sm: 6, md: 4}}>
                    <TextField fullWidth label="Email del propietario" variant="outlined" value={this.state.owner_email} onChange={(e) => this.setState({ owner_email: e.target.value })} />
                </Grid>
                <Grid size={{xs: 12, sm: 6, md: 4}}>
                    <FormControlLabel control={<Checkbox checked={this.state.explicit} onChange={(e) => this.setState({ explicit: e.target.checked })} />} label="Explicit" />
                </Grid>
                <Grid size={{xs: 12, sm: 6, md: 4}}>
                    <TextField fullWidth label="Keywords" variant="outlined" value={this.state.keywords} onChange={(e) => this.setState({ keywords: e.target.value })} />
                </Grid>
                <Grid size={12}>
                    <Button variant="contained" onClick={this.onClick}>Guardar</Button>
                </Grid>
            </Grid>
        );
    }
}

