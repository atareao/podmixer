import react from "react";
import Paper from '@mui/material/Paper';
import Box from '@mui/material/Box';
import Tab from '@mui/material/Tab';
import Tabs from '@mui/material/Tabs';
import CustomTabPanel from '../components/custom_tab_panel';
import Feed from '../components/feed';
import Telegram from '../components/telegram';
import Twitter from '../components/twitter';

interface ConfigurationPageState {
    tabIndex: number
}

function a11yProps(index: number) {
    return {
        id: `simple-tab-${index}`,
        'aria-controls': `simple-tabpanel-${index}`,
    };
}


export default class ConfigurationPage extends react.Component<{}, ConfigurationPageState> {

    constructor(props: {}) {
        super(props);
        this.state = {
            tabIndex: 0
        }
    }

    handleChange = (event: React.SyntheticEvent, newValue: number) => {
        this.setState({ tabIndex: newValue });
    };

    render = () => {
        return (
            <Paper sx={{ width: '100%', p: 2 }}>
                <h1>Configuración</h1>
                <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
                    <Tabs value={this.state.tabIndex} onChange={this.handleChange} aria-label="basic tabs example">
                        <Tab label="Feed" {...a11yProps(0)} />
                        <Tab label="Telegram" {...a11yProps(1)} />
                        <Tab label="Twitter" {...a11yProps(2)} />
                    </Tabs>
                </Box>
                <CustomTabPanel value={this.state.tabIndex} index={0}>
                    <Feed />
                </CustomTabPanel>
                <CustomTabPanel value={this.state.tabIndex} index={1}>
                    <Telegram />
                </CustomTabPanel>
                <CustomTabPanel value={this.state.tabIndex} index={2}>
                    <Twitter />
                </CustomTabPanel>
            </Paper>
        );
    }
};


