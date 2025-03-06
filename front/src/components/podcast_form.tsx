import React from 'react';
import AddIcon from '@mui/icons-material/Add';
import SaveIcon from '@mui/icons-material/Save';
import CancelIcon from '@mui/icons-material/Cancel';
import Grid from '@mui/material/Grid2';
import Button from '@mui/material/Button';
import Checkbox from '@mui/material/Checkbox';
import TextFieldWithErrors from './textfield_with_errors';
import Snackbar from '@mui/material/Snackbar';
import Alert, { AlertProps } from '@mui/material/Alert';
import { BASE_URL } from '../constants';
import { GridRowModel } from '@mui/x-data-grid';
import { AdapterDayjs } from '@mui/x-date-pickers/AdapterDayjs';
import { LocalizationProvider } from '@mui/x-date-pickers/LocalizationProvider';
import dayjs from 'dayjs';
import { DateTimePicker } from '@mui/x-date-pickers/DateTimePicker';
//import {isHttpUri, isHttpsUri} from 'valid-url';

const ENDPOINT = 'podcasts';

interface State {
    editing: boolean
    snackbar: Pick<AlertProps, 'children' | 'severity'> | null
    row: GridRowModel | null
}

interface Props {
    handleOnNew: Function
    handleOnUpdated: Function
    handleOnCancelled: Function
    row: GridRowModel | null
}

export default class PodcastForm extends React.Component<Props, State> {

    constructor(props: Props) {
        super(props);
        this.state = {
            editing: false,
            snackbar: null,
            row: props.row,
        };
    }

    render() {
        console.log("Rendering NewClassForm");
        console.log("Selected row", this.state.row);
        return (
            <React.Fragment>
                <Grid container spacing={1} mb={4}>
                    {/* --- Start fields --- */}
                    <Grid size={2}>
                        <TextFieldWithErrors
                            fullWidth
                            name="name"
                            label="Nombre"
                            helpText="Nombre"
                            value={this.state.row ? this.state.row.name : ""}
                            validations={[
                                { check: (value: string) => value.length > 0, msg: "El nombre del podcast no puede estar vacío" },
                            ]}
                            onChanged={this.handleOnChangedName}
                            onError={(error: string) => {
                                this.handleSnackbar("error", JSON.stringify(error));
                                this.setState({ row: null });

                            }}
                        />
                    </Grid>
                    <Grid size={5}>
                        <TextFieldWithErrors
                            fullWidth
                            name="url"
                            label="Url"
                            helpText="Url"
                            value={this.state.row ? this.state.row.url : ""}
                            validations={[
                                { check: (value: string) => value.length > 0, msg: "La url no puede estar vacía" },
                            ]}
                            onChanged={this.handleOnChangedUrl}
                            onError={(error: string) => {
                                this.handleSnackbar("error", JSON.stringify(error));
                                this.setState({ row: null });

                            }}
                        />
                    </Grid>
                    <Grid size={0.5}>
                        <Checkbox checked={this.state.row?.active != null ? this.state.row.active : false } onChange={(e) =>  this.setState({ row: { ...this.state.row, active: e.target.checked }})} />
                    </Grid>
                    <Grid size={2.5}>
                        <LocalizationProvider dateAdapter={AdapterDayjs}>
                            <DateTimePicker
                                label="Ultima publicación"
                                value={dayjs(this.state.row?.last_pub_date != null? this.state.row.last_pub_date: dayjs() )}
                                onChange={(newValue) => this.setState({ row: { ...this.state.row, last_pub_date:  dayjs(newValue).toDate() }})}
                            />
                        </LocalizationProvider>
                    </Grid>
                    {/* --- End fields --- */}
                    {/* --- Start actions --- */}
                    {!this.state.editing && (
                        <Grid size={1}>
                            <Button sx={{ p: 2 }} variant="contained" onClick={this.handleOnNew}><AddIcon /></Button>
                        </Grid>
                    )}
                    {this.state.editing && (
                        <>
                            <Grid>
                                <Button sx={{ p: 2 }} variant="contained" onClick={this.handleOnUpdate} ><SaveIcon /></Button>
                            </Grid>
                            <Grid>
                                <Button sx={{ p: 2 }} variant="contained" onClick={this.cancelUpdate} ><CancelIcon /></Button>
                            </Grid>
                        </>
                    )}
                    {/* --- End actions --- */}
                </Grid>
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
            </React.Fragment>
        )
    }

    handleOnNew = async (event: any) => {
        event.preventDefault();
        if (this.state.row === null) {
            console.log("Errors in form");
            this.handleSnackbar("error", "Errors in form");
            return;
        }
        const data = this.state.row;
        if (!data.active) {
            data.active = false;
        }
        if(!data.last_pub_date) {
            data.last_pub_date = dayjs();
        }
        const body = JSON.stringify(data);
        console.log(`Submitting: ${body}`);
        try {
            const response = await fetch(`${BASE_URL}/api/v1/${ENDPOINT}`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: body
            });
            const responseJson = await response.json();
            console.log("Response:", JSON.stringify(responseJson));
            if (response.ok) {
                this.setState({
                    ...this.state.row, ...responseJson.data,
                });
                this.props.handleOnNew(responseJson.data);
            } else {
                console.log("Response:", JSON.stringify(responseJson));
                this.handleSnackbar("error", JSON.stringify(responseJson.message));
            }
        } catch (error) {
            console.error('Error:', error);
            this.handleSnackbar("error", JSON.stringify(error));
        }
        this.setState({
            editing: false,
            row: null,
        });
    }

    handleOnUpdate = async (event: any) => {
        event.preventDefault();
        if (this.state.row === null) {
            console.log("Errors in form");
            this.handleSnackbar("error", "Errors in form");
            return;
        }
        const data = this.state.row;
        if (!data.active) {
            data.active = false;
        }
        if(!data.last_pub_date) {
            data.last_pub_date = dayjs();
        }
        const body = JSON.stringify(data);
        console.log(`Update body: ${body}`);
        try {
            const response = await fetch(`${BASE_URL}/api/v1/${ENDPOINT}`, {
                method: 'PATCH',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: body
            });
            console.log(`Submitting: ${body}`);
            const responseJson = await response.json();
            console.log("Response:", JSON.stringify(responseJson));
            if (response.ok) {
                this.setState({
                    ...this.state.row, ...responseJson.data,
                });
                this.props.handleOnUpdated(responseJson.data);
            } else {
                console.log("Response:", JSON.stringify(responseJson));
                this.handleSnackbar("error", JSON.stringify(responseJson.message));
            }
        } catch (error) {
            console.error('Error:', error);
            this.handleSnackbar("error", JSON.stringify(error));
        }
        this.setState({
            editing: false,
            row: null,
        });
    }

    cancelUpdate = async () => {
        console.log("Cancelling item");
        this.props.handleOnCancelled();
        this.setState({
            editing: false,
            row: null,
        });
    }

    componentDidMount = () => {
        console.log("Mounting NewClassForm");
        console.log("Selected row", this.state.row);
    }

    componentDidUpdate = (prevProps: Props) => {
        console.log("Try to update NewClassForm");
        console.log("Selected row", this.props.row);
        console.log("Previous selected row", prevProps.row);
        console.log("State row", this.state.row);
        console.log("Editing", this.state.editing);
        if (this.props.row !== null &&
            prevProps !== null &&
            (this.props.row !== prevProps.row)
        ) {
            console.log("Updating NewClassForm");
            this.setState({
                row: this.props.row,
                editing: true
            });
        }
    }

    handleCloseSnackbar = () => {
        this.setState({ snackbar: null });
    }

    handleSnackbar = (severity: AlertProps['severity'], message: AlertProps['children']) => {
        this.setState({ snackbar: { severity: severity, children: message } });
    }

    handleOnChangedName = (value: string) => {
        this.setState({ row: { ...this.state.row, name: value } });
    }
    handleOnChangedUrl = (value: string) => {
        this.setState({ row: { ...this.state.row, url: value } });
    }

    handleOnErrorName = (error: string) => {
        this.setState({ row: null });
        this.handleSnackbar("error", error);
    }
}

