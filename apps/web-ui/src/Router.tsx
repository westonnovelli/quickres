import {
	createRootRoute,
	createRoute,
	createRouter,
	Link,
	Outlet,
	RouterProvider,
	useParams,
} from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import EventNotFound from "./EventNotFound";
import EventReserve from "./_2-1_EventReserve";
import EventViewer from "./_1_EventViewer";
import Header from "./Header";
import ReservationConfirmation from "./_3_ReservationConfirmation";
import VerifyEmail from "./_4_VerifyEmail";
import RetrieveReservation from "./_5_RetrieveReservation";
import ScanReservation from "./_6_ScanReservation";
import QRCodeScanner from "./_7_QRCodeScanner";

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
					<button
						type="button"
						className="bg-blue-500 hover:bg-blue-600 text-white font-semibold py-2 px-4 rounded-lg transition-colors"
					>
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
		return (
			<>
				<Header />
				<EventViewer eventId={eventId} />
			</>
		);
	},
});

const eventReserveRoute = createRoute({
	getParentRoute: () => rootRoute,
	path: "/event/$eventId/reserve",
	errorComponent: EventNotFound,
	component: function EventReserveContainer() {
		const { eventId } = useParams({ from: "/event/$eventId/reserve" });
		return (
			<>
				<Header />
				<EventReserve eventId={eventId} />
			</>
		);
	},
});

const reservationConfirmationRoute = createRoute({
	getParentRoute: () => rootRoute,
	path: "/reservation/confirmation",
	validateSearch: (search: Record<string, unknown>) => ({
		reservationId: search.reservationId as string | undefined,
		eventId: search.eventId as string | undefined,
	}),
	component: function ReservationConfirmationContainer() {
		return <ReservationConfirmation />;
	},
});

const verifyEmailRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: "/verify/$token",
        component: function VerifyEmailContainer() {
                return (
                        <>
                                <Header />
                                <VerifyEmail />
                        </>
                );
        },
});

const retrieveReservationRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: "/retrieve/$reservationId",
        component: function RetrieveReservationContainer() {
                return (
                        <>
                                <Header />
                                <RetrieveReservation />
                        </>
                );
        },
});

const qrScannerRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: "/scan",
        component: QRCodeScanner,
});

const scanReservationRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: "/scan/$token",
        component: function ScanReservationContainer() {
                return <ScanReservation />;
        },
});

const routeTree = rootRoute.addChildren([
        indexRoute,
        eventViewerRoute,
        eventReserveRoute,
        reservationConfirmationRoute,
        verifyEmailRoute,
        retrieveReservationRoute,
        qrScannerRoute,
        scanReservationRoute,
]);

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
