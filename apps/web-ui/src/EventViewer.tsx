import { useQuery } from "@tanstack/react-query";

interface Event {
    id: string;
    name: string;
    description?: string;
    start_time: string;
    end_time: string;
    capacity: number;
    location?: string;
    created_at: string;
    updated_at: string;
    status: 'Open' | 'Full' | 'Finished';
}

interface Props {
    eventId: string;
}

const useEvent = (eventId: string) => {
    const { data, isLoading, error } = useQuery({
        queryKey: ["event", eventId],
        queryFn: () => fetch(`http://localhost:8000/events/${eventId}`).then(res => res.json()),
    });

    const event = (() => {
        if (!data) return null;
        return data as Event;
    })();

    return { event, isLoading, error };
}

const formatDateTime = (dateTimeString: string) => {
    const date = new Date(dateTimeString);
    return date.toLocaleString('en-US', {
        weekday: 'long',
        year: 'numeric',
        month: 'long',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
    });
};

const getStatusColor = (status: string) => {
    switch (status) {
        case 'Open':
            return 'bg-green-100 text-green-800 border-green-200';
        case 'Full':
            return 'bg-yellow-100 text-yellow-800 border-yellow-200';
        case 'Finished':
            return 'bg-gray-100 text-gray-800 border-gray-200';
        default:
            return 'bg-gray-100 text-gray-800 border-gray-200';
    }
};

const EventViewer: React.FC<Props> = ({ eventId }) => {
    const { event, isLoading, error } = useEvent(eventId);

    if (isLoading) return (
        <div className="flex items-center justify-center min-h-screen">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
        </div>
    );
    
    if (error) throw error;
    if (!event) return <div className="text-center text-gray-500">Event not found</div>;

    return (
        <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 py-6 px-4">
            <div className="max-w-6xl mx-auto">
                {/* Header */}
                <div className="bg-white rounded-xl shadow-lg p-6 mb-6">
                    <div className="flex items-start justify-between mb-4">
                        <div className="flex-1">
                            <h1 className="text-2xl font-bold text-gray-900 mb-2">{event.name}</h1>
                            {event.description && (
                                <p className="text-base text-gray-600 leading-relaxed">{event.description}</p>
                            )}
                        </div>
                        <div className={`px-3 py-1 rounded-full border-2 font-semibold text-sm ${getStatusColor(event.status)}`}>
                            {event.status}
                        </div>
                    </div>

                    {/* Event Details Grid - More Horizontal */}
                    <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
                        {/* Time Information */}
                        <div className="bg-blue-50 rounded-lg p-4">
                            <h3 className="text-base font-semibold text-blue-900 mb-3 flex items-center">
                                <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-label="Clock icon">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg>
                                Event Schedule
                            </h3>
                            <div className="space-y-2">
                                <div>
                                    <div className="text-xs font-medium text-blue-700">Start Time</div>
                                    <div className="text-blue-900 font-semibold text-sm">{formatDateTime(event.start_time)}</div>
                                </div>
                                <div>
                                    <div className="text-xs font-medium text-blue-700">End Time</div>
                                    <div className="text-blue-900 font-semibold text-sm">{formatDateTime(event.end_time)}</div>
                                </div>
                            </div>
                        </div>

                        {/* Location and Capacity */}
                        <div className="space-y-4">
                            {event.location && (
                                <div className="bg-green-50 rounded-lg p-4">
                                    <h3 className="text-base font-semibold text-green-900 mb-3 flex items-center">
                                        <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-label="Location icon">
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
                                        </svg>
                                        Location
                                    </h3>
                                    <div className="text-green-900 font-semibold text-sm">{event.location}</div>
                                </div>
                            )}

                            <div className="bg-purple-50 rounded-lg p-4">
                                <h3 className="text-base font-semibold text-purple-900 mb-3 flex items-center">
                                    <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-label="Users icon">
                                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                                    </svg>
                                    Capacity
                                </h3>
                                <div className="text-purple-900 font-semibold text-lg">{event.capacity} people</div>
                            </div>
                        </div>

                        {/* Event Metadata */}
                        <div className="bg-gray-50 rounded-lg p-4">
                            <h3 className="text-base font-semibold text-gray-900 mb-3">Event Details</h3>
                            <div className="space-y-2 text-xs text-gray-600">
                                <div className="flex items-center">
                                    <svg className="w-3 h-3 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-label="Created date">
                                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                                    </svg>
                                    Created: {new Date(event.created_at).toLocaleDateString()}
                                </div>
                                <div className="flex items-center">
                                    <svg className="w-3 h-3 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-label="Updated date">
                                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                                    </svg>
                                    Updated: {new Date(event.updated_at).toLocaleDateString()}
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