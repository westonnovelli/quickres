import {
	Link,
	Outlet,
	RouterProvider,
	createRootRoute,
	createRoute,
	createRouter,
    useParams,
} from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import EventNotFound from "./EventNotFound";
import EventViewer from "./EventViewer";

const rootRoute = createRootRoute({
	component: () => (
		<>
			<Outlet />
			<TanStackRouterDevtools />
		</>
	),
});

const indexRoute = createRoute({
	getParentRoute: () => rootRoute,
	path: "/",
	component: function Index() {
		return (
			<div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 flex items-center justify-center">
				<div className="text-center">
					<h1 className="text-4xl font-bold text-gray-900 mb-4">
						Welcome to Quick Res
					</h1>
					<p className="text-lg text-gray-600 mb-8">
						Your super basic React app is ready!
					</p>
					<button type="button" className="bg-blue-500 hover:bg-blue-600 text-white font-semibold py-2 px-4 rounded-lg transition-colors">
						Get Started
					</button>
				</div>
			</div>
		);
	},
});

const eventViewerRoute = createRoute({
	getParentRoute: () => rootRoute,
	path: "/event/$eventId",
    errorComponent: EventNotFound,
	component: function EventViewerContainer() {
		const { eventId } = useParams({ from: "/event/$eventId" });
		return <EventViewer eventId={eventId} />;
	},
});

const routeTree = rootRoute.addChildren([indexRoute, eventViewerRoute]);

export const router = createRouter({
	routeTree,
	defaultPreload: "intent",
	scrollRestoration: true,
});

declare module "@tanstack/react-router" {
	interface Register {
		router: typeof router;
	}
}
