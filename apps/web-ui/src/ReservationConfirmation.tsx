import { Link, useSearch } from "@tanstack/react-router";

const ReservationConfirmation: React.FC = () => {
	const search = useSearch({ from: "/reservation/confirmation" });
	const finalReservationId = search.reservationId;
	const finalEventId = search.eventId;

	return (
		<div className="min-h-screen bg-gradient-to-br from-green-50 to-emerald-100 flex items-center justify-center">
			<div className="max-w-md w-full bg-white rounded-lg shadow-lg p-8 text-center">
				<div className="mb-6">
					<div className="mx-auto flex items-center justify-center h-12 w-12 rounded-full bg-green-100">
						<svg
							className="h-6 w-6 text-green-600"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
							aria-hidden="true"
						>
							<title>Success checkmark</title>
							<path
								strokeLinecap="round"
								strokeLinejoin="round"
								strokeWidth={2}
								d="M5 13l4 4L19 7"
							/>
						</svg>
					</div>
				</div>
				
				<h1 className="text-2xl font-bold text-gray-900 mb-4">
					Reservation Submitted!
				</h1>
				
				<p className="text-gray-600 mb-6">
					Your reservation request has been successfully submitted. We will send you an email with a link to verify your reservation. <span className="font-semibold">Follow the instructions in the email to complete your reservation.</span>
				</p>
				
				{finalReservationId && (
					<div className="bg-gray-50 rounded-md p-4 mb-6">
						<p className="text-sm text-gray-500">Reservation ID</p>
						<p className="text-md font-mono text-gray-900">{finalReservationId}</p>
					</div>
				)}
				
				<div className="space-y-3">
					{finalEventId && (
						<Link
							to="/event/$eventId"
							params={{ eventId: finalEventId }}
							className="block w-full bg-gray-100 text-gray-700 py-2 px-4 rounded-md border border-gray-300 hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 transition-colors"
						>
							View Event Details
						</Link>
					)}
				</div>
			</div>
		</div>
	);
};

export default ReservationConfirmation; 