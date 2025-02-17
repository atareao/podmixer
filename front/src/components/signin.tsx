import React from 'react';
import TextField from '@mui/material/TextField';
import Button from '@mui/material/Button';
import Box from '@mui/material/Box';
import Grid from '@mui/material/Grid2';
import Avatar from '@mui/material/Avatar';

interface SignInProps {
    onSubmit: Function
    responseMessage: string
}

interface SignInState {
    email: string
    password: string
}

export default class SignIn extends React.Component<SignInProps, SignInState> {

    constructor(props: SignInProps) {
        console.log("Constructing sign in form");
        super(props);
        this.reset();
    }

    reset = () => {
        this.state = {
            email: "",
            password: ""
        };
    }

    handleEmailChange = (e: any) => {
        console.log("Email changed");
        this.setState({ email: e.target.value });
    }

    handlePasswordChange = (e: any) => {
        console.log("Password changed");
        this.setState({ password: e.target.value });
    }

    handleSubmit = (e: any) => {
        e.preventDefault();
        console.log("Submitting sign in form");
        const userData = {
            email: this.state.email,
            password: this.state.password,
        };
        console.log(`Submitting user data: ${JSON.stringify(userData)}`);
        this.props.onSubmit(userData); // Pass user data to onSubmit function
    }
    render = () => {
        console.log("Rendering sign in form");
        return (
            <main>
                <form onSubmit={this.handleSubmit}>
                    <Grid
                        container
                        spacing={0}
                        direction="column"
                        alignItems="center"
                        justifyContent="center"
                        sx={{ minHeight: '100vh' }}
                    >
                        <Box sx={{ m: 1 }}>
                            <Avatar
                                alt="PreEmer"
                                src="/images/logo.svg"
                                sx={{ width: 256, height: 256, mx: "auto" }} />
                        </Box>
                        <Box sx={{ m: 1 }}>
                            <TextField
                                required
                                type="email"
                                id="email"
                                label={"Email address"}
                                variant="outlined"
                                value={this.state.email}
                                onChange={this.handleEmailChange}
                                sx={{ mx: "auto" }} />
                        </Box>
                        <Box sx={{ m: 1 }}>
                            <TextField
                                required
                                type="password"
                                id="password"
                                label={'Password'}
                                variant="outlined"
                                value={this.state.password}
                                onChange={this.handlePasswordChange} />
                        </Box>
                        <Box sx={{ m: 1 }}>
                            <Button
                                variant="contained"
                                type="submit">
                                {'Sign in'}
                            </Button>
                        </Box>
                        <Box sx={{ m: 1 }}>
                            {this.props.responseMessage && <p>{this.props.responseMessage}</p>}
                        </Box>
                    </Grid>
                </form>
            </main>
        );
    }
}
