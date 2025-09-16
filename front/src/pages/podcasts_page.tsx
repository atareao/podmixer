import React from "react";
import Paper from '@mui/material/Paper';
import TextField from '@mui/material/TextField';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import { motion } from "framer-motion";
import SimpleTable from "../components/simple_table";
import { loadData } from '../common/utils';
import PodcastDialog, { Action, } from "../components/dialogs/podcast_dialog";
import Podcast from '../models/podcast';
import {
    GridRenderCellParams,
    GridRowModel,
} from '@mui/x-data-grid';
import dayjs from 'dayjs';
import { PiArrowArcRightBold } from "react-icons/pi";
import { BASE_URL } from '../constants';

const ENDPOINT = 'podcasts';

interface State {
    rows: GridRowModel[];
    dialogOpen: boolean;
    dialogAction: Action;
    podcast?: Podcast;
    columns: any[];
    isLoading: boolean;
    isSpinning: boolean;
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
            isSpinning: false,
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
                        if (params.value) {
                            const ts = new Date(params.value);
                            console.log(ts);
                        }
                        return (
                            <>
                                <TextField
                                    sx={{ width: 160, }}
                                    type="date"
                                    variant="filled"
                                    slotProps={{ input: { readOnly: true } }}
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

    handleClick = async () => {
        // Evita reinicios si ya está girando.
        if (this.state.isSpinning) {
            return;
        }
        this.setState({ isSpinning: true });
        const url = `${BASE_URL}/api/v1/podcasts/generate`;
        const response = await fetch(url, {
            method: "POST",
            headers: {
                'Content-Type': 'application/json'
            },
        });
        console.log("Response status:", response.status);
        this.setState({ isSpinning: false });
    }

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
                {this.state.dialogOpen === true && (
                    <PodcastDialog
                        dialogOpen={this.state.dialogOpen}
                        handleClose={this.handleClose}
                        action={this.state.dialogAction}
                        podcast={this.state.podcast}
                    />)}
                <Stack spacing={2}>
                    <Stack
                        direction="row"
                        alignItems="center"
                    >
                        <Typography variant="h4" component="div" sx={{ flexGrow: 1, ml: 2 }}>
                            Podcasts
                        </Typography>
                        <Button
                            onClick={this.handleClick}
                            variant="contained"
                            color="primary"
                        >
                            <motion.div
                                // Condicionalmente establece el estado de la animación.
                                animate={this.state.isSpinning ? "spin" : "stop"}
                                variants={{
                                    // Define el estado 'spin': gira 360 grados.
                                    spin: {
                                        rotate: 360,
                                        transition: {
                                            repeat: Infinity, // Repite la animación indefinidamente.
                                            duration: 1, // Una vuelta por segundo.
                                            ease: "linear",
                                        }
                                    },
                                    // Define el estado 'stop': la rotación vuelve a 0 y se detiene.
                                    stop: {
                                        rotate: 0,
                                        transition: {
                                            duration: 0.5
                                        }
                                    },
                                }}
                            >
                                <PiArrowArcRightBold />
                            </motion.div>
                        </Button>
                    </Stack>
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
