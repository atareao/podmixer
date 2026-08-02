import React from 'react';
import AuthContext from '../components/auth_context';
import { Navigate } from 'react-router';

export default class LogoutPage extends React.Component {

    static contextType = AuthContext;
    declare context: React.ContextType<typeof AuthContext>;

    constructor(props: {}) {
        console.log("Constructing logout page");
        super(props);
    }
    componentDidMount = () => {
        console.log("Doing logout");
        this.context.logout();
    }

    render = () => {
        return (
            <Navigate to="/" />
        );
    }
}

