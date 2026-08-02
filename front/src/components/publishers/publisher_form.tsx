import React from 'react';
import TextField from '@mui/material/TextField';
import Grid from '@mui/material/Grid';
import Button from '@mui/material/Button';
import Stack from '@mui/material/Stack';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import InputLabel from '@mui/material/InputLabel';
import FormControl from '@mui/material/FormControl';
import Switch from '@mui/material/Switch';
import Typography from '@mui/material/Typography';
import Chip from '@mui/material/Chip';
import OpenInNewIcon from '@mui/icons-material/OpenInNew';
import AppRegistrationIcon from '@mui/icons-material/AppRegistration';
import { Publisher, PublisherConfig } from '../../models/publisher';
import { BASE_URL } from '../../constants';

interface PublisherFormProps {
    publisher?: Publisher;
    onSave: (publisher: Partial<Publisher>) => void;
    onCancel: () => void;
    onRefresh?: () => void;
}

interface PublisherFormState {
    name: string;
    publisher_type: string;
    template: string;
    active: boolean;
    config: Record<string, string>;
}

export default class PublisherForm extends React.Component<PublisherFormProps, PublisherFormState> {
    constructor(props: PublisherFormProps) {
        super(props);
        const p = props.publisher;
        this.state = {
            name: p?.name || '',
            publisher_type: p?.publisher_type || 'telegram',
            template: p?.template || '',
            active: p?.active || false,
            config: this.configToState(p?.config || {}),
        };
    }

    configToState(config: PublisherConfig): Record<string, string> {
        return {
            bot_token: config.bot_token || '',
            chat_id: config.chat_id || '',
            message_thread_id: config.message_thread_id || '',
            client_id: config.client_id || '',
            client_secret: config.client_secret || '',
            access_token: config.access_token || '',
            refresh_token: config.refresh_token || '',
            redirect_uri: config.redirect_uri || '',
            server_url: config.server_url || '',
            access_token_mastodon: config.access_token_mastodon || '',
            homeserver_url: config.homeserver_url || '',
            room_id: config.room_id || '',
            access_token_matrix: config.access_token_matrix || '',
        };
    }

    getConfigFields(): { key: string; label: string }[] {
        switch (this.state.publisher_type) {
            case 'telegram':
                return [
                    { key: 'bot_token', label: 'Bot Token' },
                    { key: 'chat_id', label: 'Chat ID' },
                    { key: 'message_thread_id', label: 'Message Thread ID' },
                ];
            case 'x':
                return [
                    { key: 'client_id', label: 'Client ID' },
                    { key: 'client_secret', label: 'Client Secret' },
                    { key: 'redirect_uri', label: 'Redirect URI' },
                    { key: 'access_token', label: 'Access Token' },
                    { key: 'refresh_token', label: 'Refresh Token' },
                ];
            case 'mastodon':
                return [
                    { key: 'server_url', label: 'Server URL' },
                    { key: 'redirect_uri', label: 'Redirect URI' },
                    { key: 'access_token_mastodon', label: 'Access Token' },
                ];
            case 'matrix':
                return [
                    { key: 'homeserver_url', label: 'Homeserver URL' },
                    { key: 'room_id', label: 'Room ID' },
                    { key: 'access_token_matrix', label: 'Access Token' },
                ];
            default:
                return [];
        }
    }

    handleSave = () => {
        const config: PublisherConfig = {};
        for (const field of this.getConfigFields()) {
            (config as Record<string, string>)[field.key] = this.state.config[field.key] || '';
        }
        this.props.onSave({
            name: this.state.name,
            publisher_type: this.state.publisher_type as Publisher['publisher_type'],
            template: this.state.template,
            active: this.state.active,
            config,
        });
    };

    handleOAuthConnect = async (publisherId: string) => {
        const token = localStorage.getItem('token');
        try {
            const resp = await fetch(`${BASE_URL}/api/v1/publishers/${publisherId}/oauth/authorize`, {
                method: 'POST',
                headers: { Authorization: `Bearer ${token}` },
            });
            const data = await resp.json();
            if (resp.ok && data.url) {
                const popup = window.open(data.url, 'oauth', 'width=600,height=700');
                if (!popup) {
                    alert('Popup blocked! Please allow popups for this site.');
                    return;
                }
                const handleMessage = (event: MessageEvent) => {
                    if (event.data?.type === 'oauth-success' || event.data?.type === 'oauth-error') {
                        window.removeEventListener('message', handleMessage);
                        if (event.data.type === 'oauth-success') {
                            if (this.props.onRefresh) this.props.onRefresh();
                        }
                    }
                };
                window.addEventListener('message', handleMessage);
            } else {
                console.error('OAuth authorize failed:', data);
            }
        } catch (error) {
            console.error('Error during OAuth connect:', error);
        }
    };

    render() {
        const fields = this.getConfigFields();
        return (
            <Grid container spacing={2}>
                <Grid size={12}>
                    <Typography variant="h6">
                        {this.props.publisher ? 'Edit Publisher' : 'New Publisher'}
                    </Typography>
                </Grid>
                <Grid size={6}>
                    <TextField
                        fullWidth label="Name" variant="outlined"
                        value={this.state.name}
                        onChange={(e) => this.setState({ name: e.target.value })}
                    />
                </Grid>
                <Grid size={6}>
                    <FormControl fullWidth>
                        <InputLabel>Type</InputLabel>
                        <Select
                            value={this.state.publisher_type}
                            label="Type"
                            onChange={(e) => this.setState({ publisher_type: e.target.value })}
                        >
                            <MenuItem value="telegram">Telegram</MenuItem>
                            <MenuItem value="x">X (Twitter)</MenuItem>
                            <MenuItem value="mastodon">Mastodon</MenuItem>
                            <MenuItem value="matrix">Matrix</MenuItem>
                        </Select>
                    </FormControl>
                </Grid>
                {fields.map((field) => (
                    <Grid size={6} key={field.key}>
                        <TextField
                            fullWidth label={field.label} variant="outlined"
                            value={this.state.config[field.key] || ''}
                            onChange={(e) => this.setState({
                                config: { ...this.state.config, [field.key]: e.target.value }
                            })}
                            helperText={field.key === 'redirect_uri' ? 'URL que configuras en X/Mastodon para el callback OAuth' : undefined}
                        />
                    </Grid>
                ))}
                <Grid size={12}>
                    <TextField
                        multiline minRows={4} fullWidth
                        label="Template (minijinja)" variant="outlined"
                        value={this.state.template}
                        onChange={(e) => this.setState({ template: e.target.value })}
                        helperText="Variables: {{ title }}, {{ description }}, {{ url }}. Filters: |truncate(n), |word_limit(n), |strip_html"
                    />
                </Grid>
                {this.props.publisher && (
                    <Grid size={12}>
                        <Stack direction="row" spacing={2} sx={{ mt: 1 }}>
                            {this.state.publisher_type === 'x' && this.state.config.client_id && this.state.config.client_secret && (
                                <Button
                                    variant="outlined"
                                    color="primary"
                                    startIcon={<OpenInNewIcon />}
                                    onClick={() => this.handleOAuthConnect(this.props.publisher!.id)}
                                >
                                    🔗 Connect with X
                                </Button>
                            )}
                            {this.state.publisher_type === 'mastodon' && this.state.config.server_url && (
                                <>
                                    {!this.state.config.client_id ? (
                                        <Button
                                            variant="outlined"
                                            color="primary"
                                            startIcon={<AppRegistrationIcon />}
                                            onClick={() => this.handleOAuthConnect(this.props.publisher!.id)}
                                        >
                                            📝 Register App & Connect
                                        </Button>
                                    ) : !this.state.config.access_token_mastodon ? (
                                        <Button
                                            variant="outlined"
                                            color="primary"
                                            startIcon={<OpenInNewIcon />}
                                            onClick={() => this.handleOAuthConnect(this.props.publisher!.id)}
                                        >
                                            🔗 Connect Mastodon
                                        </Button>
                                    ) : (
                                        <Chip label="✅ Connected" color="success" />
                                    )}
                                </>
                            )}
                        </Stack>
                    </Grid>
                )}
                <Grid size={12}>
                    <Switch
                        checked={this.state.active}
                        onChange={(e) => this.setState({ active: e.target.checked })}
                    />
                    <Typography variant="button">Active</Typography>
                </Grid>
                <Grid size={12}>
                    <Stack direction="row" spacing={2}>
                        <Button variant="contained" onClick={this.handleSave}>Save</Button>
                        <Button variant="outlined" onClick={this.props.onCancel}>Cancel</Button>
                    </Stack>
                </Grid>
            </Grid>
        );
    }
}