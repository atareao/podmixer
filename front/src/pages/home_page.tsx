import react from "react";
import Paper from '@mui/material/Paper';
import Typography from '@mui/material/Typography';
import Grid from '@mui/material/Grid2';
import { Navigate } from 'react-router';
import Card from '@mui/material/Card';
import CardHeader from '@mui/material/CardHeader';
import CardContent from '@mui/material/CardContent';
import CardActions from '@mui/material/CardActions';
import IconButton from '@mui/material/IconButton';
import LaunchIcon from '@mui/icons-material/Launch';

export default class HomePage extends react.Component {
    render = () => {
        return (
            <Paper sx={{ p: 2, m: 2 }}>
                <Typography variant="h3">Podmixer</Typography>
                <Grid container spacing={2}>
                    <Card variant="outlined" onClick={() => <Navigate to="/podcasts" />}>
                        <CardHeader title="Podcasts" />
                        <CardContent>
                            <p>Aquí se encuentran los podcasts.</p>
                            <p>Puedes modificar los existentes o añadir nuevos.</p>
                        </CardContent>
                        <CardActions disableSpacing>
                            <IconButton href="/podcasts" >
                                <LaunchIcon />
                            </IconButton>
                        </CardActions>
                    </Card>
                    <Card variant="outlined">
                        <CardHeader title="Configuración" />
                        <CardContent>
                            <p>Aquí puedes configurar las redes sociales.</p>
                            <p>El feed, twitter y telegram (por ahora)</p>
                        </CardContent>
                        <CardActions disableSpacing>
                            <IconButton href="/configuration" >
                                <LaunchIcon />
                            </IconButton>
                        </CardActions>
                    </Card>
                    <Card variant="outlined">
                        <CardHeader title="Acerca de" />
                        <CardContent>
                            <p>Este es el acerca de</p>
                            <p>El texto es auto generado (por ahora)</p>
                        </CardContent>
                        <CardActions disableSpacing>
                            <IconButton href="/about" >
                                <LaunchIcon />
                            </IconButton>
                            </CardActions>
                    </Card>
                </Grid>
            </Paper>
        );
    }
};

