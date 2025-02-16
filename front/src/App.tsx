import react from "react";
import {
    BrowserRouter,
    Routes,
    Route,
} from "react-router";
import MainLayout from "./layouts/main_layout";
import AuthLayout from "./layouts/auth_layout";
import ProtectedLayout from "./layouts/protected_layout";
import HomePage from "./pages/home_page";
import AboutPage from "./pages/about_page";
import LoginPage from "./pages/login_page";
import LogoutPage from "./pages/logout_page";
import PodcastsPage from "./pages/podcasts_page";
import ConfigurationPage from "./pages/configuration_page";
import { AuthContextProvider } from "./components/auth_context";
import "./App.css";

export default class App extends react.Component {
    state = {
        darkMode: true,
    }

    render = () => {
        return (
            <AuthContextProvider>
                <BrowserRouter>
                    <Routes>
                        <Route path="/" element={<MainLayout />} >
                            <Route path="about" element={<AboutPage />} />
                        </Route>
                        <Route path="/" element={<ProtectedLayout />} >
                            <Route index element={<HomePage />} />
                            <Route path="podcasts" element={<PodcastsPage />} />
                            <Route path="configuration" element={<ConfigurationPage />} />
                            <Route path="logout" element={<LogoutPage />} />
                        </Route>
                        <Route path="/" element={<AuthLayout />} >
                            <Route path="login" element={<LoginPage />} />
                        </Route>
                    </Routes>
                </BrowserRouter>
            </AuthContextProvider>
        );
    }
}

