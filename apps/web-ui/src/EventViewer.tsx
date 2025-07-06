import { Link } from "@tanstack/react-router";
import { formatDateTime, useEvent } from "./useEvent";

interface Props {
	eventId: string;
}

const getStatusColor = (status: string) => {
	switch (status) {
		case "Open":
			return "bg-green-100 text-green-800 border-green-200";
		case "Full":
			return "bg-yellow-100 text-yellow-800 border-yellow-200";
		case "Finished":
			return "bg-gray-100 text-gray-800 border-gray-200";
		default:
			return "bg-gray-100 text-gray-800 border-gray-200";
	}
};

const EventViewer: React.FC<Props> = ({ eventId }) => {
	const { event, isLoading, error } = useEvent(eventId);

	if (isLoading)
		return (
			<div className="flex items-center justify-center min-h-screen">
				<div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
			</div>
		);

	if (error) throw error;
	if (!event)
		return <div className="text-center text-gray-500">Event not found</div>;

	return (
		<div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 py-1 px-2">
			<div className="max-w-6xl mx-auto">
				{/* Header */}
				<div className="bg-white rounded-xl shadow-lg p-3 mb-6">
					<div className="flex-1">
						<h1 className="text-2xl font-bold text-gray-900 mb-2">
							{event.name}
						</h1>
						{event.description && (
							<p className="text-base text-gray-600 leading-relaxed">
								{event.description}
							</p>
						)}
					</div>

					{/* Event Details Grid - More Horizontal */}
					<div className="grid grid-cols-1 lg:grid-cols-3 gap-4 mt-2">
						{/* Time Information */}
						<div className="bg-blue-50 rounded-lg">
							<h3 className="text-base font-semibold text-blue-900 flex items-center">
								<svg
									className="w-4 h-4 mr-2"
									fill="none"
									stroke="currentColor"
									viewBox="0 0 24 24"
									aria-label="Clock"
								>
									<path
										strokeLinecap="round"
										strokeLinejoin="round"
										strokeWidth={2}
										d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
									/>
								</svg>
								When
							</h3>
							<div className="space-y-2 grid grid-cols-2 gap-2">
								<div>
									<div className="text-xs font-medium text-gray-700">
										Start Time
									</div>
									<div className="text-blue-900 font-semibold text-sm">
										{formatDateTime(event.start_time)}
									</div>
								</div>
								<div>
									<div className="text-xs font-medium text-gray-700">
										End Time
									</div>
									<div className="text-blue-900 font-semibold text-sm">
										{formatDateTime(event.end_time)}
									</div>
								</div>
							</div>
						</div>

						{event.location && (
							<div className="bg-green-50 rounded-lg">
								<h3 className="text-base font-semibold text-green-900 flex items-center">
									<svg
										className="w-4 h-4 mr-2"
										fill="none"
										stroke="currentColor"
										viewBox="0 0 24 24"
										aria-label="Location"
									>
										<path
											strokeLinecap="round"
											strokeLinejoin="round"
											strokeWidth={2}
											d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z"
										/>
										<path
											strokeLinecap="round"
											strokeLinejoin="round"
											strokeWidth={2}
											d="M15 11a3 3 0 11-6 0 3 3 0 016 0z"
										/>
									</svg>
									Where
								</h3>
								<div className="text-green-900 font-semibold text-sm">
									{event.location}
								</div>
							</div>
						)}

						{/* CTA Section */}
						{event.status === "Open" && (
							<div className="bg-blue-50 rounded-lg">
								<h3 className="text-base font-semibold text-blue-900 flex items-center">
									There's still room!
								</h3>
								<p className="text-sm text-gray-600 mb-3">
									This event is about 20% full.
								</p>
								<div className="flex justify-end">
									<Link
										to="/event/$eventId/reserve"
										params={{ eventId }}
										className="bg-blue-600 hover:bg-blue-700 text-white font-semibold py-2 px-4 rounded-lg transition-colors"
									>
										Reserve your spot
									</Link>
								</div>
							</div>
						)}

						{/* Event Metadata */}
						<div className="bg-gray-50 rounded-lg">
							<div className="space-y-2 text-xs text-gray-600">
								<div className="flex items-center">
									<svg
										className="w-3 h-3 mr-2"
										fill="none"
										stroke="currentColor"
										viewBox="0 0 24 24"
										aria-label="Created"
									>
										<path
											strokeLinecap="round"
											strokeLinejoin="round"
											strokeWidth={2}
											d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
										/>
									</svg>
									Created: {new Date(event.created_at).toLocaleDateString()}
								</div>
								<div className="flex items-center">
									<svg
										className="w-3 h-3 mr-2"
										fill="none"
										stroke="currentColor"
										viewBox="0 0 24 24"
										aria-label="Updated"
									>
										<path
											strokeLinecap="round"
											strokeLinejoin="round"
											strokeWidth={2}
											d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
										/>
									</svg>
									Last Updated:{" "}
									{new Date(event.updated_at).toLocaleDateString()}
								</div>
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	);
};

export default EventViewer;
