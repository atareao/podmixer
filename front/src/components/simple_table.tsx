import * as React from "react";
import { MdAdd, MdDelete, MdEdit } from "react-icons/md";
import Button from '@mui/material/Button';
import {
    GridRowsProp,
    GridRowModesModel,
    DataGrid,
    GridColDef,
    GridRowId,
    GridRowModel,
    GridPaginationModel,
    GridToolbarContainer,
    GridActionsCellItem,
} from '@mui/x-data-grid';

export interface CustomAction {
    icon: any;
    label: string;
    onClick: (id: GridRowId) => void;
}

interface Props {
    onAdd: Function;
    onEdit: Function;
    onDelete: Function;
    customActions?: CustomAction[];
    columns: any[];
    rows: any[];
    sortBy?: string;
}

interface State {
    rows: GridRowModel[]
    rowModesModel: GridRowModesModel
    pagination: GridPaginationModel
    isLoading: boolean
}

export default class SimpleTable extends React.Component<Props, State> {
    columns: GridColDef<GridRowModel>[];
    columnGroupingModel: any[];

    constructor(props: Props) {
        super(props);
        console.log(`Constructing CustomTable:`, props);
        this.state = {
            rows: props.rows,
            rowModesModel: {},
            pagination: { page: 0, pageSize: 10 },
            isLoading: true,
        };
        this.columns = [];
        this.columnGroupingModel = [];
    }

    componentDidUpdate(prevProps: any, prevState: any) {
        console.log("Updating Simple Table");
        console.log(`prevProps:`,prevProps);
        console.log(`prevState:`, prevState.rows);
        console.log(`currentProps:`, this.props);
        console.log(`currentState:`, this.state.rows);
        if (prevProps.rows !== this.props.rows) {
            this.setState({ rows: this.props.rows });
        }
    }

    componentDidMount = async () => {
        console.log("Mounting page");
        this.setState({ isLoading: false });
        this.columns = [
            {
                field: 'actions',
                type: 'actions',
                headerName: 'Acciones',
                width: 100,
                cellClassName: 'actions',
                getActions: ({ id }) => {
                    const actions = [
                        <GridActionsCellItem
                            icon={<MdEdit />}
                            label="Edit"
                            className="textPrimary"
                            onClick={() => this.handleEditClick(id)}
                            color="inherit"
                        />,
                        <GridActionsCellItem
                            icon={<MdDelete />}
                            label="Delete"
                            onClick={() => this.handleDeleteClick(id)}
                            color="inherit"
                        />,
                    ];
                    if (this.props.customActions) {
                        this.props.customActions.forEach((action: CustomAction) => {
                            actions.push(
                                <GridActionsCellItem
                                    icon={action.icon}
                                    label={action.label}
                                    onClick={() => action.onClick(id)}
                                    color="inherit"
                                />,

                            );
                        });
                    }
                    return actions;
                },
            },
            ...this.props.columns
        ];
    }

    render = () => {

        if (this.state.isLoading) {
            return (
                <div>Loading...</div>
            );
        }
        const sortBy = this.props.sortBy?this.props.sortBy:"name";
        return (
            <>
                <DataGrid
                    initialState={{
                        sorting: {
                            sortModel: [{ field: sortBy, sort: 'asc' }]
                        },
                    }}
                    paginationModel={this.state.pagination}
                    pageSizeOptions={[10, 20, 50]}
                    disableRowSelectionOnClick
                    onPaginationModelChange={(newPaginationModel) => this.setState({ pagination: newPaginationModel })}
                    rows={this.props.rows}
                    columns={this.columns}
                    columnGroupingModel={this.columnGroupingModel}
                    editMode="row"
                    rowModesModel={this.state.rowModesModel}
                    slots={{ toolbar: this.EditToolbar }}
                />
            </>
        );
    }

    EditToolbar = () => {
        const handleClick = () => {
            const response = this.props.onAdd();
            console.log(response);
            console.log("Adding new row");
            return;
        };

        return (
            <GridToolbarContainer>
                <Button color="primary" startIcon={<MdAdd />} onClick={handleClick}/ >
            </GridToolbarContainer>
        );
    }

    handleEditClick = (id: GridRowId)  => {
        console.log("handleEditClick", id);
        const editedRow = this.state.rows.find((row) => row.id === id);
        this.props.onEdit(editedRow);
    }

    handleDeleteClick = async (id: GridRowId) => {
        console.log("Deleting row:", id);
        const deletedRow = this.state.rows.find((row) => row.id === id);
        this.props.onDelete(deletedRow);
    }

    setRows = (newRows: GridRowsProp) => {
        console.log("setRows");
        this.setState({ rows: newRows as GridRowModel[] });
    }
}

