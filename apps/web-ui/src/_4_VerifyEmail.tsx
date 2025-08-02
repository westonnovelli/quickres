import { Link, useParams, useSearch } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";

const VerifyEmail: React.FC = () => {
	const { token } = useParams({ from: "/verify/$token" });

	const { data, isLoading, error } = useQuery({
		queryKey: ["verify-email", token],
		queryFn: () =>
			fetch(`http://localhost:8000/verify/${token}`).then((res) => res.json()),
	});

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

				{isLoading ? (
					<h1 className="text-2xl font-bold text-gray-900 mb-4">
						Verifying your email...
					</h1>
				) : error || data?.error ? (
					<h1 className="text-2xl font-bold text-gray-900 mb-4">
						Error verifying email
					</h1>
				) : (
					<h1 className="text-2xl font-bold text-gray-900 mb-4">
						Email verified!
					</h1>
				)}
			</div>
		</div>
	);
};

export default VerifyEmail;
