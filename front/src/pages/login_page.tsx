import React from 'react';
import SignIn from "../components/signin";
import { Navigate } from 'react-router';
import AuthContext from '../components/auth_context';
import { BASE_URL } from '../constants';
import { jwtDecode } from 'jwt-decode';


interface LoginPageState {
    email: string
    password: string
    responseMessage: string
    redirect: boolean
}
interface UserData {
    email: string
    password: string
}

export default class LoginPage extends React.Component<{}, LoginPageState> {

    static contextType = AuthContext;
    declare context: React.ContextType<typeof AuthContext>;

    constructor(props: {}) {
        console.log("Constructing login page");
        super(props);
        this.state = {
            email: "",
            password: "",
            responseMessage: "",
            redirect: false
        };
    }

    handleSubmit = (userData: UserData) => {
        console.log("Submitting user data:", userData);
        this.setState({ email: userData.email, password: userData.password });
        this.login(userData.email, userData.password);
    };

    setToken = (token: string) => {
        localStorage.setItem('token', token);
    }

    getToken = () => {
        return localStorage.getItem('token');
    }


    login = async (email: string, password: string) => {
        console.log("Logging in user");
        try {
            console.log(`${BASE_URL}/api/v1/auth/login`);
            const response = await fetch(`${BASE_URL}/api/v1/auth/login`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    email: email,
                    password: password,
                }),
            });
            const responseJson = await response.json();
            if (response.ok) {
                this.setToken(responseJson.data.token);
                //const expirationTime = new Date(new Date().getTime() + 3600 * 1000);
                const value = this.context;
                console.log("================");
                console.log(`Value1: ${value}`);
                console.log("----------------");
                const decoded = jwtDecode(responseJson.data.token);
                console.log("Decoded:", JSON.stringify(decoded));
                console.log("----------------");
                this.context.login(responseJson.data.token);
                console.log("================");
                //LoginPage.authContext.login(responseJson.data.token);
                this.setState({
                    responseMessage: responseJson.message,
                    redirect: true,
                });
            } else {
                console.error('Login failed:', responseJson);
                this.setState({
                    responseMessage: responseJson.message,
                });
            }
        } catch (error) {
            console.error('Error:', error);
            this.setState({
                responseMessage: "Error logging in user",
            });
        }
    }


    render = () => {
        console.log("Rendering login page");
        if (this.state.redirect) {
            return (
                <Navigate to="/" />
            );
        } else {
            return (
                <SignIn
                    onSubmit={this.handleSubmit}
                    responseMessage={this.state.responseMessage}
                />
            );
        }
    }
}

