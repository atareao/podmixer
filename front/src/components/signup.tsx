import React from 'react';

interface SignUpProps {
    onSubmit: Function
    responseMessage: string
}
interface SignUpState {
    username: string
    email: string
    password: string
    confirmPassword: string
}
export default class SignUp extends React.Component<SignUpProps, SignUpState> {

    constructor(props: SignUpProps) {
        super(props);
        this.state = {
            username: "",
            email: "",
            password: "",
            confirmPassword: ""
        };
    }

    handleNameChange = (e: any) => {
        this.setState({ username: e.target.value });
    }

    handleEmailChange = (e: any) => {
        this.setState({ email: e.target.value });
    }

    handlePasswordChange = (e: any) => {
        this.setState({ password: e.target.value });
    }

    handleConfirmPasswordChange = (e: any) => {
        this.setState({ confirmPassword: e.target.value });
    }

    handleSubmit = (e: any) => {
        e.preventDefault();
        if (this.state.password !== this.state.confirmPassword) {
            alert("Password and Confirm Password do not match");
            return;
        }
        if (!this.isValidEmail(this.state.email)) {
            alert("Invalid Email Address");
            return;
        }
        // Send data to register page
        const userData = {
            username: this.state.username,
            email: this.state.email,
            password: this.state.password,
        };
        this.props.onSubmit(userData); // Pass user data to onSubmit function
    }

    isValidEmail = (email: string) => {
        // Email validation logic here (e.g., regex)
        // Return true if valid, false otherwise
        return /\S+@\S+\.\S+/.test(email);
    }

    render = () => {
        return (
            <main>
                <form onSubmit={this.handleSubmit}>
                    <h1>Please Register</h1>

                    <div>
                        <input type="text" id="username" placeholder="Your Name" value={this.state.username} onChange={this.handleNameChange} required />
                        <label htmlFor="username">Name</label>
                    </div>

                    <div>
                        <input type="email" id="email" placeholder="name@example.com" value={this.state.email} onChange={this.handleEmailChange} required />
                        <label htmlFor="email">Email address</label>
                    </div>
                    <div>
                        <input type="password" id="password" placeholder="Password" value={this.state.password} onChange={this.handlePasswordChange} required />
                        <label htmlFor="password">Password</label>
                    </div>
                    <div>
                        <input type="password" id="confirmPassword" placeholder="Confirm Password" value={this.state.confirmPassword} onChange={this.handleConfirmPasswordChange} required />
                        <label htmlFor="confirmPassword">Confirm Password</label>
                    </div>
                    <button type="submit">
                        Register
                    </button>
                    {this.props.responseMessage && <p>{this.props.responseMessage}</p>}
                </form>
            </main>
        );
    }
}


