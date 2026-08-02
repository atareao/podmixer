import React from "react";
import Dialog from "@mui/material/Dialog";
import DialogTitle from "@mui/material/DialogTitle";
import DialogActions from "@mui/material/DialogActions";
import DialogContent from "@mui/material/DialogContent";
import Stack from '@mui/material/Stack';
import TextField from '@mui/material/TextField';
import Switch from '@mui/material/Switch';
import Typography from "@mui/material/Typography";
import Button from "@mui/material/Button";
import { BASE_URL } from '../../constants';
import Box from '@mui/material/Box';
import Alert from '@mui/material/Alert';
import IconButton from '@mui/material/IconButton';
import Collapse from '@mui/material/Collapse';
import { MdClose } from "react-icons/md";
import Podcast from '../../models/podcast';
import SimpleDate from '../simple_date';

const ENDPOINT = "podcasts";

export enum Action {
    ADD = "add",
    EDIT = "edit",
    DELETE = "delete",
}

interface Props {
    dialogOpen: boolean
    handleClose: (ok: boolean, podcast: Podcast | null) => void
    action: Action
    podcast?: Podcast;
}

interface State {
    columns: any[];
    isLoading: boolean;
    podcast?: Podcast;
    messageTitle: string;
    messageMessage: string;
    messageShow: boolean;
}

export class InnerDialog extends React.Component<Props, State> {

    timer: any | null = null;

    constructor(props: Props) {
        super(props);
        console.log(`Constructing dialog for elements: ${props.podcast}`);
        const podcast = props.podcast ?? {
            name: "",
            url: "",
            active: false,
            last_pub_date: new Date(),
        } as Podcast;
        this.state = {
            columns: [],
            isLoading: true,
            podcast: podcast,
            messageTitle: "",
            messageMessage: "",
            messageShow: false,
        };
        this.timer = null;
    }

    // Se llama después de cada actualización del componente (cuando el estado o las props cambian)
    componentDidUpdate(prevProps: any, prevState: any) {
        console.log("prevProps", prevProps);
        console.log("prevState", prevState);
        if (this.props.podcast != prevProps.podcast) {
            this.setState({
                podcast: this.props.podcast,
            });
        }


        // Si el estado 'open' cambió de false a true (el collapse se abrió)
        if (!prevState.messageShow && this.state.messageShow) {
            // Limpiamos cualquier temporizador anterior por si acaso
            if (this.timer) {
                clearTimeout(this.timer);
            }
            // Configuramos el nuevo temporizador
            this.timer = setTimeout(() => {
                this.setState({ messageShow: false }); // Cierra el collapse
            }, 3000); // 2000 milisegundos = 2 segundos
        }
        // Si el estado 'open' cambió de true a false (el collapse se cerró)
        // O si el componente va a ser actualizado por alguna otra razón
        else if (prevState.messageShow && !this.state.messageShow) {
            // Si el collapse se cerró (manualmente o por temporizador), limpiamos el temporizador
            if (this.timer) {
                clearTimeout(this.timer);
                this.timer = null; // Reseteamos la referencia
            }
        }
    }

    // Se llama justo antes de que el componente sea desmontado del DOM
    componentWillUnmount() {
        // Es crucial limpiar el temporizador para evitar fugas de memoria
        if (this.timer) {
            clearTimeout(this.timer);
            this.timer = null;
        }
    }

    doWithValue = async () => {
        if(this.state.podcast === null || this.state.podcast === undefined){
            this.showMessage("Error", "Los valores no son correctos");
            return;
        }
        if(this.state.podcast.name.trim() === ""){
            this.showMessage("Error", "El nombre del podcast no puede estar vacío");
            return;
        }
        if(this.state.podcast.url.trim() === ""){
            this.showMessage("Error", "La url del podcast no puede estar vacía");
            return;
        }
        let body;
        let method;
        let url;
        if(this.props.action == Action.ADD){
            method = "POST";
            url = `${BASE_URL}/api/v1/${ENDPOINT}`;
            body = JSON.stringify({
                name: this.state.podcast?.name,
                url: this.state.podcast?.url,
                active: this.state.podcast?.active,
                last_pub_date: this.state.podcast?.last_pub_date,
                created_at: new Date(),
                updated_at: new Date(),
            });
            console.log("Body", body);
        }else if(this.props.action == Action.EDIT){
            method = "PATCH"
            url = `${BASE_URL}/api/v1/${ENDPOINT}`;
            body = JSON.stringify({
                id: this.state.podcast?.id,
                name: this.state.podcast?.name,
                url: this.state.podcast?.url,
                active: this.state.podcast?.active,
                last_pub_date: this.state.podcast?.last_pub_date,
                created_at: this.state.podcast?.created_at,
                updated_at: this.state.podcast?.updated_at,
            });
            console.log("Body", body);
        }else if(this.props.action == Action.DELETE){
            method = "DELETE";
            if(this.props.podcast?.id){
                url = `${BASE_URL}/api/v1/${ENDPOINT}?id=${this.props.podcast.id}`;
            }
            body = null;
        }
        if(!url){
            return;
        }
        try {
            const response = await fetch(url, {
                method: method,
                headers: {
                    'Content-Type': 'application/json'
                },
                body: body
            });
            console.log(`Submitting: ${body}`);
            const responseJson = await response.json();
            console.log("Response:", JSON.stringify(responseJson));
            if (response.ok) {
                console.log("Response:", JSON.stringify(responseJson.data));
                this.props.handleClose(true, responseJson.data);
            } else {
                this.showMessage(`Error: ${response.status}`, response.statusText);
            }
        } catch (error) {
            console.error('Error:', error);
            this.showMessage("Error", String(error));
        }
    }

    showMessage(title: string, message: string) {
        console.log("Showing message:", title, message);
        this.setState({
            messageTitle: title,
            messageMessage: message,
            messageShow: true,
        })
    }

    handleClose = async (ok: boolean) => {
        console.log("Closing dialog with action", ok);
        if (!ok) {
            this.props.handleClose(ok, null);
            return;
        }
        await this.doWithValue();
    }

    render = () => {
        console.log("Rendering dialog");
        console.log(`dialogOpen: ${this.props.dialogOpen}`);
        let dialogTitle = "";
        if (this.props.action == Action.ADD) {
            dialogTitle = "Nuevo Podcast";
        } else if (this.props.action == Action.EDIT) {
            dialogTitle = "Editar Podcast";
        } else if (this.props.action == Action.DELETE) {
            dialogTitle = "Eliminar Podcast";
        }
        return (
            <>
                <Dialog
                    onClose={this.handleClose}
                    open={this.props.dialogOpen}
                >
                    <DialogTitle>{dialogTitle}</DialogTitle>
                    <DialogContent>
                        <Box sx={{ width: '100%' }}>
                            <Collapse in={this.state.messageShow}>
                                <Alert
                                    action={
                                        <IconButton
                                            aria-label="close"
                                            color="inherit"
                                            size="small"
                                            onClick={() => {
                                                this.setState({ messageShow: false });
                                            }}
                                        >
                                            <MdClose fontSize="inherit" />
                                        </IconButton>
                                    }
                                    sx={{ mb: 2 }}
                                >
                                    {this.state.messageMessage}
                                </Alert>
                            </Collapse>
                        </Box>
                        <Stack
                            spacing={2}
                            sx={{ width: "500px" }}
                        >
                            <Stack direction="row" spacing={2} alignItems="center">
                                <Typography sx={{ minWidth: 60 }}>Nombre:</Typography>
                                <TextField
                                    fullWidth
                                    variant="outlined"
                                    value={this.state.podcast?.name ?? ""}
                                    slotProps={{ input: {readOnly: this.props.action === Action.DELETE } }}
                                    onChange={(e) => {
                                        this.setState({
                                            podcast: {
                                                ...this.state.podcast,
                                                name: e.target.value,
                                            } as Podcast
                                        });
                                    }
                                    }
                                />
                            </Stack>
                            <Stack direction="row" spacing={2} alignItems="center">
                                <Typography sx={{ minWidth: 60 }}>Url:</Typography>
                                <TextField
                                    fullWidth
                                    variant="outlined"
                                    value={this.state.podcast?.url ?? ""}
                                    slotProps={{ input: {readOnly: this.props.action === Action.DELETE } }}
                                    onChange={(e) => {
                                        this.setState({
                                            podcast: {
                                                ...this.state.podcast,
                                                url: e.target.value,
                                            } as Podcast
                                        });
                                    }
                                    }
                                />
                            </Stack>
                            <Stack direction="row" spacing={2} alignItems="center">
                                <Typography sx={{ minWidth: 60 }}>Ultima publicación:</Typography>
                                <SimpleDate
                                    variant="outlined"
                                    value={this.state.podcast?.last_pub_date}
                                    readOnly={this.props.action === Action.DELETE}
                                    onChange={(value: any) => {
                                        this.setState({
                                            podcast: {
                                                ...this.state.podcast,
                                                last_pub_date: new Date(value)
                                            } as Podcast
                                        });
                                    }}
                                />
                            </Stack>
                            <Stack direction="row" spacing={2} alignItems="center">
                                <Typography sx={{ minWidth: 100 }}>Activo:</Typography>
                                <Switch
                                    checked={this.state.podcast?.active}
                                    disabled={this.props.action === Action.DELETE }
                                    onChange={(e) => {
                                        this.setState({
                                            podcast: {
                                                ...this.state.podcast,
                                                active: e.target.checked
                                            } as Podcast
                                        });
                                    }}
                                />
                            </Stack>
                        </Stack>
                    </DialogContent>
                    <DialogActions>
                        <Button onClick={() => this.handleClose(true)} variant="contained">
                            Aceptar
                        </Button>
                        <Button onClick={() => this.handleClose(false)} variant="contained">
                            Cancelar
                        </Button>
                    </DialogActions>
                </Dialog>
            </>

        );
    };
}

interface ExternalProps {
    dialogOpen: boolean;
    handleClose: (ok: boolean, podcast: Podcast | null) => void
    action: Action;
    podcast?: Podcast
}
export default function PodcastDialog(props: ExternalProps) {
    console.log("ValueDialog", props);
    return (
        <InnerDialog
            dialogOpen={props.dialogOpen}
            handleClose={props.handleClose}
            action={props.action}
            podcast={props.podcast}
        />
    );
}

