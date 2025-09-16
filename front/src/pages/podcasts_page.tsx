import React from "react";
import Paper from '@mui/material/Paper';
import TextField from '@mui/material/TextField';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';
import SimpleTable from "../components/simple_table";
import { loadData } from '../common/utils';
import PodcastDialog, {Action, } from "../components/dialogs/podcast_dialog";
import Podcast from '../models/podcast';
import {
    GridRenderCellParams,
    GridRowModel,
} from '@mui/x-data-grid';
import dayjs from 'dayjs';

const ENDPOINT = 'podcasts';

interface State {
    rows: GridRowModel[];
    dialogOpen: boolean;
    dialogAction: Action;
    podcast?: Podcast;
    columns: any[];
    isLoading: boolean;
}

export default class PodcastsPage extends React.Component<{}, State> {

    constructor(props: {}) {
        super(props);
        console.log("Constructing page");
        this.state = {
            rows: [],
            dialogOpen: false,
            dialogAction: Action.ADD,
            columns: [],
            isLoading: true,
        };
    }

    loadData = async () => {
        console.log("Loading data");
        const responseJson = await loadData(ENDPOINT);
        if (responseJson.status === 200) {
            console.log(`Data loaded: JSON: ${JSON.stringify(responseJson)}`);
            return responseJson.data;
        }
        return [];
    }

    onAdd() {
        console.log("Add new element");
        this.setState({
            dialogOpen: true,
            dialogAction: Action.ADD,
            podcast: undefined,
        });
    }

    onEdit(editedItem: any) {
        console.log("Edit element", editedItem);
        this.setState({
            dialogOpen: true,
            dialogAction: Action.EDIT,
            podcast: { ...this.state.podcast, ...editedItem }
        });
    }

    onDelete(editedItem: any) {
        console.log("Delete element", editedItem);
        this.setState({
            dialogOpen: true,
            dialogAction: Action.DELETE,
            podcast: { ...this.state.podcast, ...editedItem }
        });
    }

    componentDidUpdate(prevProps: any, prevState: any) {
        console.log("Updating diptychs_page");
        console.log(`prevProps:`, prevProps);
        console.log(`prevState:`, prevState.rows);
        console.log(`currentProps:`, this.props);
        console.log(`currentState:`, this.state.rows);
    }

    componentDidMount = async () => {
        console.log("Mounting page");
        const rows = await this.loadData();
        this.setState({
            isLoading: false,
            rows: rows,
            columns: [
                {
                    field: "name",
                    headerName: "Nombre",
                    type: "string",
                    width: 200,
                },
                {
                    field: "url",
                    headerName: "Url",
                    type: "string",
                    width: 600,
                },
                {
                    field: "active",
                    headerName: "Active",
                    type: "boolean",
                    width: 100,
                },
                {
                    field: "last_pub_date",
                    headerName: "Ultima publicación",
                    type: "dste",
                    width: 250,
                    renderCell: (params: GridRenderCellParams<any, string>) => {
                        console.log(params.value);
                        if(params.value) {
                            const ts = new Date(params.value);
                            console.log(ts);
                        }
                        return (
                        <>
                            <TextField
                                sx={{ width: 160, }}
                                type="date"
                                variant="filled"
                                slotProps={{ input: {readOnly: true } }}
                                value={dayjs(params.value).format("YYYY-MM-DD")}
                            />
                            </>
                        );
                    }
                },
            ],
        });
    }

    handleClose = (ok: boolean, podcast: Podcast | null) => {
        console.log("Closing dialog with action:", ok);
        console.log("State", this.state.dialogAction);
        console.log(podcast);
        if (ok && podcast != null) {
            if (this.state.dialogAction === Action.ADD) {
                console.log("Adding new podcast", podcast);
                this.setState({ rows: [...this.state.rows, podcast] });
            } else if (this.state.dialogAction === Action.DELETE) {
                console.log("Deleting podcast", podcast);
                this.setState({
                    rows: this.state.rows.filter((row) => row.id !== podcast.id),
                });
            } else if (this.state.dialogAction === Action.EDIT) {
                console.log("Editing podcast", podcast);
                console.log(` podcast: ${podcast.id}`);
                this.setState({
                    rows: this.state.rows.map((row) =>
                        row.id === podcast.id ? podcast : row
                    ),
                });
            }
        }
        this.setState({ dialogOpen: false });
    };

    render = () => {
        console.log("Rendering dyptichs_page");
        if (this.state.isLoading) {
            return (
                <>
                    <Paper sx={{ width: "100%", p: 2 }}>
                        <h1>Elements</h1>
                        <div>Loading...</div>
                    </Paper>
                </>
            );
        }
        console.log(`Dialog is open ${this.state.dialogOpen}`);
        console.log("Estado de podcast:", this.state.podcast);
        return (
            <>
                { this.state.dialogOpen === true && (
                <PodcastDialog
                    dialogOpen={this.state.dialogOpen}
                    handleClose={this.handleClose}
                    action={this.state.dialogAction}
                    podcast={this.state.podcast}
                />)}
                <Stack spacing={2}>
                    <Typography variant="h4" component="div" sx={{ flexGrow: 1, ml: 2 }}>
                    Podcasts
                    </Typography>
                                    <SimpleTable
                                        onAdd={this.onAdd.bind(this)}
                                        onEdit={this.onEdit.bind(this)}
                                        onDelete={this.onDelete.bind(this)}
                                        columns={this.state.columns}
                                        rows={this.state.rows}
                                    />
                </Stack>
            </>
        );
    };
}
