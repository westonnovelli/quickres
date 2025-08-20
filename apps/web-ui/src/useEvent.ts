import { useQuery } from "@tanstack/react-query";

// TODO needs to come from codegen
interface Event {
	id: string;
	name: string;
	description?: string;
	start_time: string;
	end_time: string;
        capacity: number;
        max_spots_per_reservation: number;
        location?: string;
        created_at: string;
        updated_at: string;
        status: "Open" | "Full" | "Finished";
}

// would be could to make this more precise
interface Result {
	event: Event | null;
	isLoading: boolean;
	error: Error | null;
}

export const useEvent = (eventId: string): Result => {
	const { data, isLoading, error } = useQuery({
		queryKey: ["event", eventId],
		queryFn: () =>
			fetch(`http://localhost:8000/events/${eventId}`).then((res) =>
				res.json(),
			),
	});

	const event = (() => {
		if (!data) return null;
		return data as Event;
	})();

	return { event, isLoading, error };
};

export const formatDateTime = (dateTimeString: string) => {
	const date = new Date(dateTimeString);
	return date.toLocaleString("en-US", {
		weekday: "long",
		year: "numeric",
		month: "long",
		day: "numeric",
		hour: "2-digit",
		minute: "2-digit",
	});
};