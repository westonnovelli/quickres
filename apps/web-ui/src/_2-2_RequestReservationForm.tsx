import { useForm, Field } from "@tanstack/react-form";
import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { router } from "./Router";

interface Props {
	eventId: string;
	maxSpotCount: number;
}

interface ReserveRequest {
	event_id: string;
	user_name: string;
	user_email: string;
	spot_count: number;
}

interface ReserveResponse {
	reservation_id: string;
	status: "Pending" | "Confirmed";
}

const RequestReservationForm: React.FC<Props> = ({ eventId, maxSpotCount }) => {
	const [isSubmitting, setIsSubmitting] = useState(false);

	// Create mutation for reservation
	const reservationMutation = useMutation({
		mutationFn: async (data: ReserveRequest): Promise<ReserveResponse> => {
			const response = await fetch(`http://localhost:8000/reserve`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify(data),
			});

			if (!response.ok) {
				throw new Error(`Failed to submit reservation: ${response.statusText}`);
			}

			return response.json();
		},
		onSuccess: (data) => {
			console.log("Reservation submitted successfully:", data);

			// Navigate to confirmation page with reservation data
			router.navigate({
				to: "/reservation/confirmation",
				search: {
					reservationId: data.reservation_id,
					eventId: eventId,
				},
			});
		},
		onError: (error) => {
			console.error("Error submitting reservation:", error);
		},
	});

	const form = useForm({
		defaultValues: {
			user_name: "",
			user_email: "",
			spot_count: 1,
		},
		onSubmit: async ({ value }) => {
			setIsSubmitting(true);
			try {
				await reservationMutation.mutateAsync({
					event_id: eventId,
					...value,
				});
			} finally {
				setIsSubmitting(false);
			}
		},
	});

	return (
		<form
			onSubmit={(e) => {
				e.preventDefault();
				e.stopPropagation();
				form.handleSubmit();
			}}
			className="space-y-4"
		>
			<div>
				<label
					htmlFor="name"
					className="block text-sm font-medium text-gray-700 mb-1"
				>
					Name
				</label>
				<Field
					form={form}
					name="user_name"
					validators={{
						onBlur: ({ value }) => {
							if (!value) return "Name is required";
							if (typeof value === "string" && value.length < 2) {
								return "Name must be at least 2 characters";
							}
							return undefined;
						},
					}}
				>
					{(field) => (
						<>
							<input
								id="user_name"
								type="text"
								placeholder="Enter your name"
								className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
								value={field.state.value as string}
								onBlur={field.handleBlur}
								onChange={(e) => field.handleChange(e.target.value)}
							/>
							{field.state.meta.errors && (
								<p className="mt-1 text-sm text-red-600">
									{field.state.meta.errors.join(", ")}
								</p>
							)}
						</>
					)}
				</Field>
			</div>

			<div>
				<label
					htmlFor="email"
					className="block text-sm font-medium text-gray-700 mb-1"
				>
					Email
				</label>
				<Field
					form={form}
					name="user_email"
					validators={{
						onBlur: ({ value }) => {
							if (!value) return "Email is required";
							// TODO could use zod to validate email
							const emailRegex = /^[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}$/i;
							if (typeof value === "string" && !emailRegex.test(value)) {
								return "Please enter a valid email address";
							}
							return undefined;
						},
					}}
				>
					{(field) => (
						<>
							<input
								id="user_email"
								type="email"
								placeholder="Enter your email"
								className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
								value={field.state.value as string}
								onBlur={field.handleBlur}
								onChange={(e) => field.handleChange(e.target.value)}
							/>
							{field.state.meta.errors && (
								<p className="mt-1 text-sm text-red-600">
									{field.state.meta.errors.join(", ")}
								</p>
							)}
						</>
					)}
				</Field>
			</div>

			<div>
				<label
					htmlFor="spot_count"
					className="block text-sm font-medium text-gray-700 mb-1"
				>
					Number of spots (max {maxSpotCount})
				</label>
				<Field
					form={form}
					name="spot_count"
					validators={{
						onBlur: ({ value }) => {
							if (!value) {
								form.setFieldValue("spot_count", 1);
							}
							// TODO could use zod to validate email
							const emailRegex = /^[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}$/i;
							if (typeof value === "string" && !emailRegex.test(value)) {
								return "Please enter a valid email address";
							}
							return undefined;
						},
					}}
				>
					{(field) => (
						<>
							<input
								id="spot_count"
								type="number"
								min={1}
								max={maxSpotCount}
								placeholder="1"
								className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
								value={field.state.value}
								onBlur={field.handleBlur}
								onChange={(e) => field.handleChange(e.target.valueAsNumber)}
							/>
							{field.state.meta.errors && (
								<p className="mt-1 text-sm text-red-600">
									{field.state.meta.errors.join(", ")}
								</p>
							)}
						</>
					)}
				</Field>
			</div>

			{reservationMutation.isError && (
				<div className="p-3 bg-red-50 border border-red-200 rounded-md">
					<p className="text-sm text-red-600">
						{reservationMutation.error instanceof Error
							? reservationMutation.error.message
							: "An error occurred while submitting your reservation"}
					</p>
				</div>
			)}

			<button
				type="submit"
				disabled={
					isSubmitting || !form.state.isValid || reservationMutation.isPending
				}
				className="w-full bg-blue-600 text-white py-2 px-4 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed"
			>
				{isSubmitting || reservationMutation.isPending
					? "Requesting..."
					: "Request Reservation"}
			</button>
		</form>
	);
};

export default RequestReservationForm;
