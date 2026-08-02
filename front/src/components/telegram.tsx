import React from 'react';
import TextField from '@mui/material/TextField';
import Grid from '@mui/material/Grid';
import Button from '@mui/material/Button';
import Checkbox from '@mui/material/Checkbox';
import Typography from '@mui/material/Typography';
import { BASE_URL } from '../constants';

interface TelegramState {
    token: string
    chatId: string
    threadId: string
    template: string
    active: boolean
}

export default class Telegram extends React.Component<{}, TelegramState> {
    constructor(props: {}) {
        super(props);
        this.state = {
            token: "",
            chatId: "",
            threadId: "",
            template: "",
            active: false
        }
    }

    onClick = async () => {
        console.log("Submitting data");
        console.log(this.state.template);
        const body = `{
                    "token": ${this.state.token ? JSON.stringify(this.state.token) : "\"\""},
                    "chat_id": ${this.state.chatId ? JSON.stringify(this.state.chatId) : "\"\""},
                    "thread_id": ${this.state.threadId ? JSON.stringify(this.state.threadId) : "\"\""},
                    "template": ${this.state.template ? JSON.stringify(this.state.template) : "\"\""},
                    "active": ${JSON.stringify(this.state.active)}
                }`;
        try {
            const response = await fetch(`${BASE_URL}/api/v1/config/telegram`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: body
            });
            console.log(`Submitting: ${body}`);
            const responseJson = await response.json();
            console.log("Response:", JSON.stringify(responseJson));
            if (response.ok) {
                console.log("Response:", JSON.stringify(responseJson));
                const data = responseJson.data;
                this.setState({
                    token: data.token,
                    chatId: data.chat_id,
                    threadId: data.thread_id,
                    template: data.template,
                    active: data.active
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

    loadData = async () => {
        console.log("Loading data");
        try {
            const response = await fetch(`${BASE_URL}/api/v1/config/telegram`, {
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
                    token: data.token,
                    chatId: data.chat_id,
                    threadId: data.thread_id,
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
                <Grid size={4}>
                    <TextField
                        fullWidth
                        helperText="Token de telegram"
                        label="Token"
                        variant="outlined"
                        value={this.state.token}
                        onChange={(e) => this.setState({ token: e.target.value })}
                    />
                </Grid>
                <Grid size={4}>
                    <TextField
                        fullWidth
                        helperText="Chat id de telegram"
                        label="Chat id"
                        variant="outlined"
                        value={this.state.chatId}
                        onChange={(e) => this.setState({ chatId: e.target.value })}
                    />
                </Grid>
                <Grid size={4}>
                    <TextField
                        fullWidth
                        helperText="Thread id de telegram"
                        label="Thread id"
                        variant="outlined"
                        value={this.state.threadId}
                        onChange={(e) => this.setState({ threadId: e.target.value })}
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

