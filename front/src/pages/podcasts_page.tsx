import React from "react";
import Paper from '@mui/material/Paper';
import Toolbar from '@mui/material/Toolbar';
import EditIcon from '@mui/icons-material/Edit';
import DeleteIcon from '@mui/icons-material/Delete';
import { loadData } from '../common/utils';
import PodcastForm from '../components/podcast_form';
import {
    GridRowModesModel,
    DataGrid,
    GridColDef,
    GridActionsCellItem,
    GridRowId,
    GridRowModel,
    GridPaginationModel,
} from '@mui/x-data-grid';
import Snackbar from '@mui/material/Snackbar';
import Alert, { AlertProps } from '@mui/material/Alert';

import { BASE_URL } from '../constants';
const ENDPOINT = 'podcasts';

interface State {
    rows: GridRowModel[]
    rowModesModel: GridRowModesModel
    pagination: GridPaginationModel
    selectedRow: GridRowModel | null
    isLoading: boolean
    snackbar: Pick<AlertProps, 'children' | 'severity'> | null
}

export default class PodcastsPage extends React.Component<{}, State> {

    columns: GridColDef<GridRowModel>[];
    constructor(props: {}) {
        super(props);
        console.log("Constructing page");
        this.state = {
            rows: [],
            rowModesModel: {},
            pagination: { page: 1, pageSize: 10 },
            selectedRow: null,
            isLoading: true,
            snackbar: null,
        };
        this.columns = [];
    }

    componentDidMount = async () => {
        console.log("Mounting page");
        await this.loadData();
        this.setState({ isLoading: false });
        this.columns = [
            { field: 'name', headerName: 'Nombre', type: 'string', width: 180 },
            { field: 'url', headerName: 'Url', type: 'string', width: 470, editable: true },
            { field: 'active', headerName: 'Activo', type: 'boolean', width: 60, editable: true },
            {
                field: 'last_pub_date',
                headerName: 'Ultima publicación',
                type: 'dateTime',
                width: 220,
                editable: true,
                valueGetter: (value) => value && new Date(value),
            },
            {
                field: 'actions',
                type: 'actions',
                headerName: 'Actions',
                width: 100,
                cellClassName: 'actions',
                getActions: ({ id }) => {
                    return [
                        <GridActionsCellItem
                            icon={<EditIcon />}
                            label="Edit"
                            className="textPrimary"
                            onClick={this.handleEditClick(id)}
                            color="inherit"
                        />,
                        <GridActionsCellItem
                            icon={<DeleteIcon />}
                            label="Delete"
                            onClick={this.handleDeleteClick(id)}
                            color="inherit"
                        />,
                    ];
                },
            },
        ];
    }

    render = () => {
        if (this.state.isLoading) {
            return (
                <>
                    <Toolbar />
                    <Paper sx={{ width: '100%', p: 2 }}>
                        <h1>Subclasses</h1>
                        <div>Loading...</div>
                    </Paper>
                </>
            );
        }
        return (
            <>
                <Toolbar />
                <Paper sx={{ width: '100%', p: 2 }}>
                    <h1>Podcasts</h1>
                    <PodcastForm
                        handleOnNew={this.onNew}
                        handleOnUpdated={this.onUpdate}
                        handleOnCancelled={this.onCancelled}
                        row={this.state.selectedRow}
                    />
                    <DataGrid
                        initialState={{
                            sorting: {
                                sortModel: [{ field: 'last_pub_date', sort: 'desc' }]
                            },
                        }}
                        paginationModel={this.state.pagination}
                        pageSizeOptions={[10]}
                        onPaginationModelChange={(newPaginationModel) => this.setState({ pagination: newPaginationModel })}
                        rows={this.state.rows}
                        columns={this.columns}
                        editMode="row"
                        rowModesModel={this.state.rowModesModel}
                        onRowModesModelChange={this.handleRowModesModelChange}
                    />
                    {!!this.state.snackbar && (
                        <Snackbar
                            open
                            anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}
                            onClose={this.handleCloseSnackbar}
                            autoHideDuration={2000}
                        >
                            <Alert {...this.state.snackbar} onClose={this.handleCloseSnackbar} />
                        </Snackbar>
                    )}
                </Paper>
            </>
        );
    }

    loadData = async () => {
        console.log("Loading data");
        const responseJson = await loadData(ENDPOINT);
        if (responseJson.status === 200) {
            console.log(`Data loaded: JSON: ${JSON.stringify(responseJson)}`);
            this.setState({
                rows: responseJson.data,
                pagination: { ...this.state.pagination, page: 0 },
            });
        }
    }

    onNew = async (row: GridRowModel) => {
        console.log("New row:", row);
        this.setState({
            rows: [...this.state.rows.filter((row) => row.id !== -1), row],
            selectedRow: null
        });
    }

    onUpdate = async (updatedRow: GridRowModel) => {
        console.log('Updated row:', updatedRow);
        this.setState({ rows: [...this.state.rows.filter((row) => row.id !== updatedRow.id), updatedRow] });

    }

    onCancelled = () => {
        console.log("Cancelled");
        this.setState({ selectedRow: null });
    }

    handleEditClick = (id: GridRowId) => async () => {
        console.log("Edit:", id);
        this.state.rows.some((row) => {
            if (row.id === id) {
                console.log("Editing:", row);
                this.setState({ selectedRow: row });
                return true;
            }
        });
    }

    handleDeleteClick = (id: GridRowId) => async () => {
        try {
            const response = await fetch(`${BASE_URL}/api/v1/${ENDPOINT}?id=${id}`, {
                method: 'DELETE',
                headers: {
                    'Content-Type': 'application/json',
                },
            });
            if (response.ok) {
                this.setState({ rows: this.state.rows.filter((row) => row.id !== id) });
            }
        } catch (error) {
            console.error('Error:', error);
        }
    };

    handleRowModesModelChange = (newRowModesModel: GridRowModesModel) => {
        this.setState({ rowModesModel: newRowModesModel });
    };

    handleCloseSnackbar = () => {
        this.setState({ snackbar: null });
    }

    handleSnackbar = (severity: AlertProps['severity'], message: AlertProps['children']) => {
        this.setState({ snackbar: { severity: severity, children: message } });
    }
}
