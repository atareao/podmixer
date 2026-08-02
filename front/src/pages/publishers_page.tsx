import React from 'react';
import Paper from '@mui/material/Paper';
import Box from '@mui/material/Box';
import Tab from '@mui/material/Tab';
import Tabs from '@mui/material/Tabs';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import DialogContent from '@mui/material/DialogContent';
import CustomTabPanel from '../components/custom_tab_panel';
import PublisherList from '../components/publishers/publisher_list';
import PublisherForm from '../components/publishers/publisher_form';
import PublisherLogs from '../components/publishers/publisher_logs';
import { Publisher, PublishLog } from '../models/publisher';
import { BASE_URL } from '../constants';

interface PublishersPageState {
    publishers: Publisher[];
    logs: PublishLog[];
    tab: number;
    dialogOpen: boolean;
    editingPublisher: Publisher | undefined;
    loading: boolean;
}

export default class PublishersPage extends React.Component<{}, PublishersPageState> {
    constructor(props: {}) {
        super(props);
        this.state = {
            publishers: [],
            logs: [],
            tab: 0,
            dialogOpen: false,
            editingPublisher: undefined,
            loading: false,
        };
    }

    componentDidMount = () => {
        this.loadPublishers();
        this.loadLogs();
    };

    loadPublishers = async () => {
        try {
            const token = localStorage.getItem('token');
            const response = await fetch(`${BASE_URL}/api/v1/publishers`, {
                headers: { 'Authorization': `Bearer ${token}` },
            });
            const json = await response.json();
            if (response.ok) {
                this.setState({ publishers: json.data || [] });
            }
        } catch (error) {
            console.error('Error loading publishers:', error);
        }
    };

    loadLogs = async () => {
        try {
            const token = localStorage.getItem('token');
            const response = await fetch(`${BASE_URL}/api/v1/publishers/logs`, {
                headers: { 'Authorization': `Bearer ${token}` },
            });
            const json = await response.json();
            if (response.ok) {
                this.setState({ logs: json.data || [] });
            }
        } catch (error) {
            console.error('Error loading logs:', error);
        }
    };

    handleSave = async (publisher: Partial<Publisher>) => {
        const token = localStorage.getItem('token');
        const isEdit = !!this.state.editingPublisher;
        const url = isEdit
            ? `${BASE_URL}/api/v1/publishers/${this.state.editingPublisher!.id}`
            : `${BASE_URL}/api/v1/publishers`;
        const method = isEdit ? 'PATCH' : 'POST';

        try {
            const response = await fetch(url, {
                method,
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${token}`,
                },
                body: JSON.stringify(publisher),
            });
            if (response.ok) {
                this.setState({ dialogOpen: false, editingPublisher: undefined });
                this.loadPublishers();
            }
        } catch (error) {
            console.error('Error saving publisher:', error);
        }
    };

    handleTest = async (id: string) => {
        const token = localStorage.getItem('token');
        try {
            const response = await fetch(`${BASE_URL}/api/v1/publishers/${id}/test`, {
                method: 'POST',
                headers: { 'Authorization': `Bearer ${token}` },
            });
            const json = await response.json();
            console.log('Test result:', json);
            this.loadLogs();
        } catch (error) {
            console.error('Error testing publisher:', error);
        }
    };

    handleToggle = async (id: string) => {
        const token = localStorage.getItem('token');
        try {
            await fetch(`${BASE_URL}/api/v1/publishers/${id}/toggle`, {
                method: 'POST',
                headers: { 'Authorization': `Bearer ${token}` },
            });
            this.loadPublishers();
        } catch (error) {
            console.error('Error toggling publisher:', error);
        }
    };

    handleDelete = async (id: string) => {
        const token = localStorage.getItem('token');
        try {
            await fetch(`${BASE_URL}/api/v1/publishers/${id}`, {
                method: 'DELETE',
                headers: { 'Authorization': `Bearer ${token}` },
            });
            this.loadPublishers();
        } catch (error) {
            console.error('Error deleting publisher:', error);
        }
    };

    render() {
        return (
            <Box sx={{ width: '100%', p: 2 }}>
                <Paper sx={{ width: '100%', mb: 2, p: 2 }}>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 2 }}>
                        <Tabs value={this.state.tab} onChange={(_, v) => this.setState({ tab: v })}>
                            <Tab label="Publishers" />
                            <Tab label="Logs" />
                        </Tabs>
                        <Button
                            variant="contained"
                            onClick={() => this.setState({ dialogOpen: true, editingPublisher: undefined })}
                        >
                            Add Publisher
                        </Button>
                    </Box>

                    <CustomTabPanel value={this.state.tab} index={0}>
                        <PublisherList
                            publishers={this.state.publishers}
                            onEdit={(p) => this.setState({ dialogOpen: true, editingPublisher: p })}
                            onTest={this.handleTest}
                            onToggle={this.handleToggle}
                            onDelete={this.handleDelete}
                        />
                    </CustomTabPanel>

                    <CustomTabPanel value={this.state.tab} index={1}>
                        <PublisherLogs logs={this.state.logs} />
                    </CustomTabPanel>
                </Paper>

                <Dialog open={this.state.dialogOpen} onClose={() => this.setState({ dialogOpen: false })} maxWidth="md" fullWidth>
                    <DialogContent>
                        <PublisherForm
                            publisher={this.state.editingPublisher}
                            onSave={this.handleSave}
                            onCancel={() => this.setState({ dialogOpen: false })}
                        />
                    </DialogContent>
                </Dialog>
            </Box>
        );
    }
}