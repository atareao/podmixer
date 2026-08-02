import React from 'react';
import { DataGrid, GridColDef } from '@mui/x-data-grid';
import Button from '@mui/material/Button';
import Stack from '@mui/material/Stack';
import Switch from '@mui/material/Switch';
import { Publisher } from '../../models/publisher';

interface PublisherListProps {
    publishers: Publisher[];
    onEdit: (publisher: Publisher) => void;
    onTest: (id: string) => void;
    onToggle: (id: string) => void;
    onDelete: (id: string) => void;
}

export default class PublisherList extends React.Component<PublisherListProps> {
    columns: GridColDef[] = [
        { field: 'name', headerName: 'Name', flex: 1 },
        { field: 'publisher_type', headerName: 'Type', width: 120 },
        {
            field: 'active',
            headerName: 'Active',
            width: 100,
            renderCell: (params) => (
                <Switch
                    checked={params.value}
                    onChange={() => this.props.onToggle(params.row.id)}
                />
            ),
        },
        {
            field: 'actions',
            headerName: 'Actions',
            width: 250,
            sortable: false,
            renderCell: (params) => (
                <Stack direction="row" spacing={1}>
                    <Button size="small" onClick={() => this.props.onEdit(params.row)}>Edit</Button>
                    <Button size="small" color="success" onClick={() => this.props.onTest(params.row.id)}>Test</Button>
                    <Button size="small" color="error" onClick={() => this.props.onDelete(params.row.id)}>Delete</Button>
                </Stack>
            ),
        },
    ];

    render() {
        return (
            <div style={{ height: 400, width: '100%' }}>
                <DataGrid
                    rows={this.props.publishers}
                    columns={this.columns}
                    getRowId={(row) => row.id}
                    disableRowSelectionOnClick
                />
            </div>
        );
    }
}