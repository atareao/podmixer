import react from "react";
import Paper from '@mui/material/Paper';
import Feed from '../components/feed';

interface ConfigurationPageState {
}



export default class ConfigurationPage extends react.Component<{}, ConfigurationPageState> {

    render = () => {
        return (
            <Paper sx={{ width: '100%', p: 2 }}>
                <h1>Configuración</h1>
                <Feed />
            </Paper>
        );
    }
};


