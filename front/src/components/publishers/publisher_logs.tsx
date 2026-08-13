import React from 'react';
import { DataGrid, GridColDef } from '@mui/x-data-grid';
import Chip from '@mui/material/Chip';
import { PublishLog } from '../../models/publisher';

interface PublisherLogsProps {
    logs: PublishLog[];
}

export default class PublisherLogs extends React.Component<PublisherLogsProps> {
    columns: GridColDef[] = [
        { field: 'publisher_name', headerName: 'Publisher', width: 150 },
        { field: 'publisher_type', headerName: 'Type', width: 100 },
        { field: 'episode_title', headerName: 'Episode', flex: 1 },
        {
            field: 'status',
            headerName: 'Status',
            width: 120,
            renderCell: (params) => {
                const colorMap: Record<string, 'success' | 'error' | 'warning' | 'info'> = {
                    success: 'success',
                    error: 'error',
                    'dry-run': 'info',
                };
                const color = colorMap[params.value as string] ?? 'warning';
                return <Chip label={params.value} color={color} size="small" />;
            },
        },
        { field: 'message', headerName: 'Message', flex: 1 },
        { field: 'created_at', headerName: 'Date', width: 180 },
    ];

    render() {
        return (
            <div style={{ height: 400, width: '100%' }}>
                <DataGrid
                    rows={this.props.logs}
                    columns={this.columns}
                    getRowId={(row) => row.id}
                    disableRowSelectionOnClick
                />
            </div>
        );
    }
}