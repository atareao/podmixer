import React from "react";
import Paper from '@mui/material/Paper';
import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Card from '@mui/material/Card';
import CardHeader from '@mui/material/CardHeader';
import CardContent from '@mui/material/CardContent';
import SimpleTable from "../components/simple_table";
import { loadData } from '../common/utils';
import PodcastDialog, {Action, } from "../components/dialogs/podcast_dialog";
import Podcast from '../models/podcast';
import {
    GridRowModel,
} from '@mui/x-data-grid';

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
                    haeaderName: "Nombre",
                    typs: "string",
                    "width": 150,
                    editable: true,
                },
                {
                    field: "url",
                    haeaderName: "Url",
                    typs: "string",
                    "width": 400,
                    editable: true,
                },
                {
                    field: "active",
                    haeaderName: "Active",
                    typs: "boolean",
                    "width": 150,
                    editable: true,
                },
                {
                    field: "last_pub_date",
                    haeaderName: "Ultima publicación",
                    typs: "date",
                    "width": 150,
                    editable: true,
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
                    <Box sx={{ height: "100px" }} />
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
                <Box style={{ height: 100 }} />
                { this.state.dialogOpen === true && (
                <PodcastDialog
                    dialogOpen={this.state.dialogOpen}
                    handleClose={this.handleClose}
                    action={this.state.dialogAction}
                    podcast={this.state.podcast}
                />)}
                <Stack spacing={2}>
                    <Card variant="outlined">
                        <CardHeader title="Podcasts" />
                        <CardContent>
                            <Card
                                sx={{
                                    p: 2,
                                    display: "flex",
                                    gap: "20px",
                                    flexWrap: "wrap",
                                    margin: "auto",
                                }}
                            >
                                <SimpleTable
                                    onAdd={this.onAdd.bind(this)}
                                    onEdit={this.onEdit.bind(this)}
                                    onDelete={this.onDelete.bind(this)}
                                    columns={this.state.columns}
                                    rows={this.state.rows}
                                />
                            </Card>
                        </CardContent>
                    </Card>
                </Stack>
            </>
        );
    };
}
