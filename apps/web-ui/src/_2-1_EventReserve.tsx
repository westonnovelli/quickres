import RequestReservationForm from "./_2-2_RequestReservationForm";
import { formatDateTime, useEvent } from "./useEvent";

interface Props {
	eventId: string;
}

const EventReserve: React.FC<Props> = ({ eventId }) => {
	const { event, isLoading, error } = useEvent(eventId);

	if (isLoading) return <div>Loading...</div>;
	if (error) {
		throw error;
	}

	if (!event) {
		return <div className="text-center text-gray-500">Event not found</div>;
	}

	return (
		<div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 py-1 px-2">
			<div className="max-w-6xl mx-auto">
				<h1 className="text-2xl font-bold text-gray-900 mb-2">
					Reserve a spot for
				</h1>
					<p className="text-blue-900 font-semibold text-sm">
						{event.name}
					</p>
                    <p className="text-gray-600 text-sm">When: <span className="font-semibold">{formatDateTime(event.start_time)}</span></p>
					<p className="text-gray-600 text-sm">Where: <span className="font-semibold">{event.location}</span></p>
                                <RequestReservationForm eventId={eventId} maxSpotCount={event.max_spots_per_reservation} />
                        </div>
                </div>
        );
};

export default EventReserve;
