import react from "react";
import {
    BrowserRouter,
    Routes,
    Route,
} from "react-router";
import { CircularProgress, Box } from "@mui/material";
import MainLayout from "./layouts/main_layout";
import AuthLayout from "./layouts/auth_layout";
import ProtectedLayout from "./layouts/protected_layout";
import { AuthContextProvider } from "./components/auth_context";
import "./App.css";

const HomePage = react.lazy(() => import("./pages/home_page"));
const AboutPage = react.lazy(() => import("./pages/about_page"));
const LoginPage = react.lazy(() => import("./pages/login_page"));
const LogoutPage = react.lazy(() => import("./pages/logout_page"));
const PodcastsPage = react.lazy(() => import("./pages/podcasts_page"));
const ConfigurationPage = react.lazy(() => import("./pages/configuration_page"));
const PublishersPage = react.lazy(() => import("./pages/publishers_page"));

const suspenseFallback = (
    <Box sx={{ display: "flex", justifyContent: "center", mt: 4 }}>
        <CircularProgress />
    </Box>
);

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
                            <Route path="about" element={<react.Suspense fallback={suspenseFallback}><AboutPage /></react.Suspense>} />
                        </Route>
                        <Route path="/" element={<ProtectedLayout />} >
                            <Route index element={<react.Suspense fallback={suspenseFallback}><HomePage /></react.Suspense>} />
                            <Route path="podcasts" element={<react.Suspense fallback={suspenseFallback}><PodcastsPage /></react.Suspense>} />
                            <Route path="configuration" element={<react.Suspense fallback={suspenseFallback}><ConfigurationPage /></react.Suspense>} />
                            <Route path="publishers" element={<react.Suspense fallback={suspenseFallback}><PublishersPage /></react.Suspense>} />
                            <Route path="logout" element={<react.Suspense fallback={suspenseFallback}><LogoutPage /></react.Suspense>} />
                        </Route>
                        <Route path="/" element={<AuthLayout />} >
                            <Route path="login" element={<react.Suspense fallback={suspenseFallback}><LoginPage /></react.Suspense>} />
                        </Route>
                    </Routes>
                </BrowserRouter>
            </AuthContextProvider>
        );
    }
}

