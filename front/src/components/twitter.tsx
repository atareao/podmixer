import React from 'react';
import TextField from '@mui/material/TextField';
import Grid from '@mui/material/Grid2';
import Button from '@mui/material/Button';
import Checkbox from '@mui/material/Checkbox';
import Typography from '@mui/material/Typography';
import { BASE_URL } from '../constants';

interface TwitterState {
    clientId: string
    clientSecret: string
    accessToken: string
    refreshToken: string
    template: string
    active: boolean
}

export default class Twitter extends React.Component<{}, TwitterState> {
    constructor(props: {}) {
        super(props);
        this.state = {
            clientId: "",
            clientSecret: "",
            accessToken: "",
            refreshToken: "",
            template: "",
            active: false,
        }
    }

    onClick = async () => {
        console.log("Submitting data");
        console.log(this.state.template.replace(/\n/gm, '\\n'));
        try {
            const body = `{
                "active": ${JSON.stringify(this.state.active)},
                "client_id": ${this.state.clientId?JSON.stringify(this.state.clientId):"\"\""},
                "client_secret": ${this.state.clientSecret?JSON.stringify(this.state.clientSecret):"\"\""},
                "access_token": ${this.state.accessToken?JSON.stringify(this.state.accessToken):"\"\""},
                "refresh_token": ${this.state.refreshToken?JSON.stringify(this.state.refreshToken):"\"\""},
                "template": ${this.state.template?JSON.stringify(this.state.template):"\"\""}
            }`;
            console.log(`Submitting data1: ${body}`);
            const response = await fetch(`${BASE_URL}/api/v1/config/twitter`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: body
            });
            const responseJson = await response.json();
            console.log(`Respose: ${JSON.stringify(responseJson)}`);
            if (response.ok) {
                console.log("Response:", JSON.stringify(responseJson));
                const data = responseJson.data;
                this.setState({
                    clientId: data.client_id,
                    clientSecret: data.client_secret,
                    accessToken: data.access_token,
                    refreshToken: data.refresh_token,
                    template: data.template,
                    active: data.active,
                });
           } else {
               this.loadData();
           }
        } catch (error) {
            console.error('Error:', error);
            this.loadData();
        }
    }

    componentDidMount = () => {
        console.log("Mounting page");
        this.loadData();
    }

    loadData = async () => {
        console.log("Loading data");
        try {
            const response = await fetch(`${BASE_URL}/api/v1/config/twitter`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                },
            });
            const responseJson = await response.json();
            if (response.ok) {
                console.log("Response:", JSON.stringify(responseJson));
                const data = responseJson.data;
                this.setState({
                    clientId: data.client_id,
                    clientSecret: data.client_secret,
                    accessToken: data.access_token,
                    refreshToken: data.refresh_token,
                    template: data.template,
                    active: data.active,
                });
            }
        } catch (error) {
            console.error('Error:', error);
        }
    }
    render = () => {
        return (
            <Grid container spacing={1}>
                <Grid size={16}>
                    <Checkbox checked={this.state.active} onChange={(e) => this.setState({ active: e.target.checked })} />
                    <Typography variant="button">Active</Typography>
                </Grid>
                <Grid size={6}>
                    <TextField
                        fullWidth
                        helperText="Client id de twitter"
                        label="Client id"
                        variant="outlined"
                        value={this.state.clientId}
                        onChange={(e) => this.setState({ clientId: e.target.value })}
                    />
                </Grid>
                <Grid size={6}>
                    <TextField
                        fullWidth
                        helperText="Client secret de twitter"
                        label="Client secret"
                        variant="outlined"
                        value={this.state.clientSecret}
                        onChange={(e) => this.setState({ clientSecret: e.target.value })}
                    />
                </Grid>
                <Grid size={6}>
                    <TextField
                        fullWidth
                        helperText="Access token de twitter"
                        label="Access token"
                        variant="outlined"
                        value={this.state.accessToken}
                        onChange={(e) => this.setState({ accessToken: e.target.value })}
                    />
                </Grid>
                <Grid size={6}>
                    <TextField
                        fullWidth
                        helperText="Refresh token de twitter"
                        label="Refresh token"
                        variant="outlined"
                        value={this.state.refreshToken}
                        onChange={(e) => this.setState({ refreshToken: e.target.value })}
                    />
                </Grid>
                <Grid size={16}>
                    <TextField
                        multiline
                        minRows={8}
                        fullWidth
                        helperText="Plantilla de telegram"
                        label="Plantilla"
                        variant="outlined"
                        value={this.state.template}
                        onChange={(e) => this.setState({ template: e.target.value })}
                    />
                </Grid>
                <Grid size={16}>
                    <Button variant="contained" onClick={this.onClick}>Guardar</Button>
                </Grid>
            </Grid>
        );
    }
}

