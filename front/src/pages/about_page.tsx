import react from "react";
import Paper from '@mui/material/Paper';

export default class AboutPage extends react.Component {
    render = () => {
        return (
             <Paper sx={{ height: 400, width: '100%', p: 2 }}>
                <h1>About Page</h1>
                <p>
                    Lorem ipsum dolor sit amet, consectetur adipiscing elit. Integer commodo
                    gravida tincidunt. Nunc fringilla blandit tortor, id accumsan nisi
                    dictum quis. Aenean porttitor at mi id semper. Donec mattis bibendum
                    leo, interdum ullamcorper lectus ultrices vel. Fusce nec enim faucibus
                    nunc porta egestas. Fusce dapibus lobortis tincidunt.
                </p>
            </Paper>
        );
    }
};

