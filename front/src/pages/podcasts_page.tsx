import React from "react";
import dayjs from 'dayjs';
import Paper from '@mui/material/Paper';
import Button from '@mui/material/Button';
import AddIcon from '@mui/icons-material/Add';
import EditIcon from '@mui/icons-material/Edit';
import DeleteIcon from '@mui/icons-material/Delete';
import SaveIcon from '@mui/icons-material/Save';
import CloseIcon from '@mui/icons-material/Close';
import Grid from '@mui/material/Grid2';
import TextField from '@mui/material/TextField';
import Checkbox from '@mui/material/Checkbox';
import { AdapterDayjs } from '@mui/x-date-pickers/AdapterDayjs';
import { LocalizationProvider } from '@mui/x-date-pickers/LocalizationProvider';
import { DateTimePicker } from '@mui/x-date-pickers/DateTimePicker';
import {
    GridRowModesModel,
    GridRowModes,
    DataGrid,
    GridColDef,
    GridActionsCellItem,
    GridEventListener,
    GridRowId,
    GridRowModel,
    GridRowEditStopReasons,
    GridPaginationModel,
} from '@mui/x-data-grid';
import Snackbar from '@mui/material/Snackbar';
import Alert, { AlertProps } from '@mui/material/Alert';

import { BASE_URL } from '../constants';

interface PageState {
    rows: GridRowModel[]
    rowModesModel: GridRowModesModel
    snackbar: Pick<AlertProps, 'children' | 'severity'> | null
    pagination: GridPaginationModel
    name: string
    url: string
    active: boolean
    last_pub_date: Date
}


export default class PodcastsPage extends React.Component<{}, PageState> {
    constructor(props: {}) {
        super(props);
        console.log("Constructing page");
        this.state = {
            rows: [],
            rowModesModel: {},
            snackbar: null,
            pagination: { page: 1, pageSize: 10 },
            name: "",
            url: "",
            active: false,
            last_pub_date: new Date(),
        };
    }

    loadData = async () => {
        console.log("Loading data");
        try {
            const response = await fetch(`${BASE_URL}/api/v1/podcasts`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                },
            });
            const responseJson = await response.json();
            if (response.ok) {
                this.setState({
                    rows: responseJson.data,
                    pagination: { page: 0, pageSize: 10 }
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

    saveRow = async (row: GridRowModel) => {
        console.log('saveRow', row);
        this.setState({ rows: [...this.state.rows.filter((row) => row.id !== -1), row] });
        this.handleSnackbar('success', 'Item created');
    }

    updateRow = async (newRow: GridRowModel, oldRow: GridRowModel) => {
        console.log('updateRow', newRow);
        try {
            const response = await fetch(`${BASE_URL}/api/v1/podcasts`, {
                method: 'PATCH',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(({
                    id: newRow.id,
                    name: newRow.name,
                    hired: newRow.hired,
                })),
            });
            const responseJson = await response.json();
            let row = null;
            if (response.ok) {
                console.log('responseJson', responseJson.data);
                row = responseJson.data;
                this.handleSnackbar('success', 'Item updated');
            } else {
                row = oldRow;
                this.handleSnackbar('error', 'Item update failed');
            }
            this.setState({ rows: this.state.rows.map((r) => (r.id === row.id ? row : r)) });
        } catch (error) {
            console.error('Error:', error);
        }
    }

    columns: GridColDef<GridRowModel>[] = [
        { field: 'name', headerName: 'Nombre', type: 'string', width: 175, editable: true },
        { field: 'url', headerName: 'Url', type: 'string', width: 500, editable: true },
        { field: 'active', headerName: 'Activo', type: 'boolean', width: 80, editable: true },
        {
            field: 'last_pub_date',
            headerName: 'Ultima publicación',
            type: 'dateTime',
            width: 200,
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
                const isInEditMode = this.state.rowModesModel[id]?.mode === GridRowModes.Edit;

                if (isInEditMode) {
                    return [
                        <GridActionsCellItem
                            icon={<SaveIcon />}
                            label="Save"
                            sx={{
                                color: 'primary.main',
                            }}
                            onClick={this.handleSaveClick(id)}
                        />,
                        <GridActionsCellItem
                            icon={<CloseIcon />}
                            label="Cancel"
                            className="textPrimary"
                            onClick={this.handleCancelClick(id)}
                            color="inherit"
                        />,
                    ];
                }
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

    handleRowEditStop: GridEventListener<'rowEditStop'> = (params, event) => {
        if (params.reason === GridRowEditStopReasons.rowFocusOut) {
            event.defaultMuiPrevented = true;
        }
    };

    handleEditClick = (id: GridRowId) => () => {
        console.log('handleEditClick', id);
        this.setState({ rowModesModel: { ...this.state.rowModesModel, [id]: { mode: GridRowModes.Edit } } });
    };

    handleSaveClick = (id: GridRowId) => () => {
        console.log('handleSaveClick', id);
        const row = this.state.rows.filter((row) => row.id === id);
        console.log('row', row);
        this.setState({ rowModesModel: { ...this.state.rowModesModel, [id]: { mode: GridRowModes.View } } });
    };

    handleDeleteClick = (id: GridRowId) => async () => {
        try {
            const response = await fetch(`${BASE_URL}/api/v1/podcasts?id=${id}`, {
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

    handleCancelClick = (id: GridRowId) => () => {
        this.setState({ rowModesModel: { ...this.state.rowModesModel, [id]: { mode: GridRowModes.View, ignoreModifications: true } } });

        const editedRow = this.state.rows.find((row: GridRowModel) => row.id === id);
        if (editedRow!.isNew) {
            this.setState({ rows: this.state.rows.filter((row) => row.id !== id) });
        }
    };

    processRowUpdate = (newRow: GridRowModel, oldRow: GridRowModel) => {
        console.log('processRowUpdate', newRow);
        console.log('isNew', newRow.isNew);
        console.log('oldRow', oldRow);
        if (newRow.isNew === true) {
            this.saveRow(newRow);
        } else {
            this.updateRow(newRow, oldRow);
        }
        const updatedRow = { ...newRow, isNew: false };
        this.setState({ rows: this.state.rows.map((row) => (row.id === newRow.id ? row : row)) });
        return updatedRow;
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
    onClick = async () => {
        console.log("Submitting feed");
        const body = `{
                    "name": ${this.state.name ? JSON.stringify(this.state.name) : "\"\""},
                    "url": ${this.state.url ? JSON.stringify(this.state.url) : "\"\""},
                    "last_pub_date": ${this.state.last_pub_date ? JSON.stringify(this.state.last_pub_date) : "\"\""},
                    "active": ${JSON.stringify(this.state.active)}
                }`;
        try {
            const response = await fetch('http://localhost:3000/api/v1/podcasts', {
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
                    name: data.name,
                    url: data.url,
                    active: data.active,
                    last_pub_date: data.last_pub_date,
                });
                this.saveRow(data);
            }
        } catch (error) {
            console.error('Error:', error);
        }
    }

    render = () => {
        return (
            <Paper sx={{ width: '100%', p: 2 }}>
                <h1>Podcasts</h1>
                <Grid container spacing={1} mb={4}>
                    <Grid size={2}>
                        <TextField
                            fullWidth
                            helperText="Nombre"
                            label="Nombre"
                            variant="outlined"
                            value={this.state.name}
                            onChange={(e) => this.setState({ name: e.target.value })}
                        />
                    </Grid>
                    <Grid size={5}>
                        <TextField
                            fullWidth
                            helperText="Url"
                            label="url"
                            variant="outlined"
                            value={this.state.url}
                            onChange={(e) => this.setState({ url: e.target.value })}
                        />
                    </Grid>
                    <Grid size={0.5}>
                        <Checkbox checked={this.state.active} onChange={(e) => this.setState({ active: e.target.checked })} />
                    </Grid>
                    <Grid size={3}>
                        <LocalizationProvider dateAdapter={AdapterDayjs}>
                            <DateTimePicker
                                label="Ultima publicación"
                                value={dayjs(this.state.last_pub_date)}
                                onChange={(newValue) => this.setState({ last_pub_date: dayjs(newValue).toDate() })}
                            />
                        </LocalizationProvider>
                    </Grid>
                    <Grid size={1}>
                        <Button sx={{ p: 2 }} variant="contained" onClick={this.onClick}><AddIcon /></Button>
                    </Grid>
                </Grid>
                <DataGrid
                    paginationModel={this.state.pagination}
                    pageSizeOptions={[10]}
                    onPaginationModelChange={(newPaginationModel) => this.setState({ pagination: newPaginationModel })}
                    rows={this.state.rows}
                    columns={this.columns}
                    editMode="row"
                    rowModesModel={this.state.rowModesModel}
                    onRowModesModelChange={this.handleRowModesModelChange}
                    onRowEditStop={this.handleRowEditStop}
                    processRowUpdate={this.processRowUpdate}
                //slots={{ toolbar: EditToolbar }}
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
        );
    }
};

