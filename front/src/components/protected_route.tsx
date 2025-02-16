import React from 'react';
import { Navigate, Outlet } from 'react-router';
import AuthContext from './auth_context';

export default class ProtectedRoute extends React.Component {
    static contextType = AuthContext;
    declare context: React.ContextType<typeof AuthContext>;

    render() {
        const value = this.context;
        if(!value.isLoggedIn) {
            return <Navigate to="/login" />;
        }
        return <Outlet />;
    }
}

