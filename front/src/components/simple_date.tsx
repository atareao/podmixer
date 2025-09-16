import React from 'react';
import TextField from '@mui/material/TextField';
import dayjs from 'dayjs';
import utc from 'dayjs/plugin/utc';

dayjs.extend(utc);


interface Props {
    variant?: "filled" | "outlined" | "standard"
    value?: Date | null
    onChange?: Function
    readOnly?: boolean
}

interface State {
    value: Date | null
}

export default class SimpleDate extends React.Component<Props, State> {

    constructor(props: Props) {
        super(props);
        this.state = {
            value: props.value ? props.value : null,
        }
    }

    handleChange = (event: any) => {
        const date = new Date(event.target.value);
        const utcDate = Date.UTC(
            date.getFullYear(), date.getMonth(), date.getDate(), 0,0,0);
        console.log(date, utcDate);
        this.setState({value: date});
        this.props.onChange ? this.props.onChange(date) : null;
    }

    render = () => {
        console.log("Rendering date", this.state.value);
        return (
            <TextField
                sx={{ width: 160 }}
                type="date"
                slotProps={{ input: {readOnly: this.props.readOnly ?? false } }}
                variant={this.props.variant ? this.props.variant : "standard"}
                value={this.state.value?dayjs(this.state.value).format("YYYY-MM-DD"): ""}
                onChange={this.handleChange}
            />
        );
    }
}


