import { cn } from "@/lib/utils";
import { Check } from "lucide-react";
import { FC, HTMLAttributes } from "react";

interface PricingCardProps extends HTMLAttributes<HTMLDivElement> {
	plan: string;
	price: string;
	features: string[];
	popular?: boolean;
}

export const PricingCard: FC<PricingCardProps> = ({
	plan,
	price,
	features,
	popular = false,
	className,
	...props
}) => {
	return (
		<div
			className={cn(
				"relative flex w-full flex-col rounded-xl border-2 border-black bg-white p-6",
				"dark:border-white dark:bg-black",
				{
					"ring-4 ring-purple-500": popular,
				},
				className,
			)}
			{...props}
		>
			{popular && (
				<div className="absolute -top-4 left-1/2 -translate-x-1/2 rounded-full bg-purple-500 px-4 py-1 text-white">
					Popular
				</div>
			)}
			<h2 className="text-2xl font-bold">{plan}</h2>
			<p className="mt-4 text-4xl font-bold">{price}</p>
			<ul className="mt-6 space-y-2">
				{features.map((feature) => (
					<li key={feature} className="flex items-center gap-2">
						<Check className="h-6 w-6 text-green-500" />
						<span>{feature}</span>
					</li>
				))}
			</ul>
		</div>
	);
};
