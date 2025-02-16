import * as React from 'react';
import Box from '@mui/material/Box';

interface TabPanelProps {
    children?: React.ReactNode;
    index: number;
    value: number;
}

export default class CustomTabPanel extends React.Component<TabPanelProps> {
    constructor(props: TabPanelProps) {
        super(props);
    }

    render = () => {
        return (
            <div
                role="tabpanel"
                hidden={this.props.value !== this.props.index}
                id={`simple-tabpanel-${this.props.index}`}
                aria-labelledby={`simple-tab-${this.props.index}`}
            >
                {this.props.value === this.props.index && <Box sx={{ p: 3 }}>{this.props.children}</Box>}
            </div>
        );
    }
}
