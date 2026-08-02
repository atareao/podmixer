import React from 'react';
import TextField from '@mui/material/TextField';
import { Validation } from '../common/types';

interface State {
    value: string | number | null
    error: string | null
}

interface Props {
    fullWidth?: boolean,
    value: string | number | null,
    name: string,
    label: string,
    helpText: string,
    validations: Validation[]
    onChanged: Function
    onError: Function
}

export default class TextFieldWithErrors extends React.Component<Props, State> {
    constructor(props: Props) {
        super(props);
        this.state = {
            value: props.value? props.value : "",
            error: null,
        }
    }
    componentDidUpdate = (prevProps: Props) => {
        console.log("componentDidUpdate", prevProps, this.props);
        if(prevProps.value !== this.props.value){
            console.log("Updating value");
            const value = this.props.value? this.props.value : "";
            this.setState({value: value});
        }
    }


    render = () => {
        console.log(`Rendering TextFieldWithErrors: ${this.state.value}`);
        return (
            <React.Fragment>
                <TextField
                    fullWidth={this.props.fullWidth}
                    name="name"
                    label={this.state.error ? "Error" : this.props.label}
                    variant="outlined"
                    onChange={this.handleChangeEvent}
                    onBlur={this.handleOnBlur}
                    value={this.state.value?this.state.value:""}
                    helperText={this.state.error ? this.state.error : this.props.helpText}
                    error={this.state.error !== null}
                />
            </React.Fragment>
        )
    }

    handleOnBlur = (event: React.FocusEvent<HTMLInputElement>) => {
        for(const validation of this.props.validations){
            const msg = (validation.check(event.target.value)) ? "" : validation.msg;
            if (msg !== "") {
                this.setState({
                    error: msg,
                    value: "",
                });
                this.props.onError(msg);
                return;
            }
        }
        this.setState({
            error: null,
            value: event.target.value,
        });
    }
    handleChangeEvent = (event: React.ChangeEvent<HTMLInputElement>) => {
        event.preventDefault();
        console.log(`target: ${event.target}`);
        console.log(`value: ${event.target.value}`);
        console.log(`value: ${event.target.name}`);
        for(const validation of this.props.validations){
            const msg = (validation.check(event.target.value)) ? "" : validation.msg;
            if (msg !== "") {
                this.setState({
                    error: msg,
                    value: "",
                });
                this.props.onError(msg);
                return;
            }
        }
        this.setState({
            error: null,
            value: event.target.value,
        });
        this.props.onChanged(event.target.value);
    }
}

