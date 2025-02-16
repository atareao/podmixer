import react from "react";
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Grid from '@mui/material/Grid2';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';

export default class HomePage extends react.Component {
    render = () => {
        return (
            <Box>
                <Typography variant="h1">Narrow Page</Typography>
                <p>
                    Lorem ipsum dolor sit amet, consectetur adipiscing elit. Integer commodo
                    gravida tincidunt. Nunc fringilla blandit tortor, id accumsan nisi
                    dictum quis. Aenean porttitor at mi id semper. Donec mattis bibendum
                    leo, interdum ullamcorper lectus ultrices vel. Fusce nec enim faucibus
                    nunc porta egestas. Fusce dapibus lobortis tincidunt.
                </p>
                <Grid  container spacing={2}>
                    <Card>
                        <CardContent>
                            Card
                        </CardContent>
                    </Card>
                    <Card>
                        <CardContent>
                            Card
                        </CardContent>
                    </Card>
                </Grid>
                <p>
                    Lorem ipsum dolor sit amet, consectetur adipiscing elit. Integer commodo
                    gravida tincidunt. Nunc fringilla blandit tortor, id accumsan nisi
                    dictum quis. Aenean porttitor at mi id semper. Donec mattis bibendum
                    leo, interdum ullamcorper lectus ultrices vel. Fusce nec enim faucibus
                    nunc porta egestas. Fusce dapibus lobortis tincidunt.
                </p>
            </Box>
        );
    }
};

