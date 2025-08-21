import React, { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../components/ui/dialog';
import { Button } from '../components/ui/button';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
import { FundFlow, FundVerseBackend } from '../lib/ic';
import { formatE8s, formatCurrency } from '../lib/utils';
import { Loader2, TrendingUp } from 'lucide-react';

const contributionSchema = z.object({
  amount: z.string().min(1, 'Amount is required'),
}).refine((data) => {
  const amount = parseFloat(data.amount);
  return amount > 0 && amount <= 1000; // Max 1000 ICP
}, {
  message: 'Amount must be between 0.00000001 and 1000 ICP',
});

type ContributionForm = z.infer<typeof contributionSchema>;

interface ContributionDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  campaignId: bigint;
  campaignTitle: string;
  fundFlowActor: any;
  backendActor: any;
  onContributionSuccess: () => void;
}

export const ContributionDialog: React.FC<ContributionDialogProps> = ({
  open,
  onOpenChange,
  campaignId,
  campaignTitle,
  fundFlowActor,
  backendActor,
  onContributionSuccess,
}) => {
  const [isLoading, setIsLoading] = useState(false);

  const {
    register,
    handleSubmit,
    formState: { errors },
    reset,
    watch,
  } = useForm<ContributionForm>({
    resolver: zodResolver(contributionSchema),
  });

  const amount = watch('amount');
  const amountE8s = amount ? Math.floor(parseFloat(amount) * 100_000_000) : 0;

  const onSubmit = async (data: ContributionForm) => {
    setIsLoading(true);
    try {
      // First, register the user if not already registered
      try {
        await fundFlowActor.register_user('Anonymous User', 'user@example.com');
      } catch (error) {
        // User might already be registered, continue
        console.log('User registration skipped:', error);
      }

      // Make the ICP contribution
      const contributionId = await fundFlowActor.contribute_icp(
        backendActor.getCanisterPrincipal(),
        campaignId,
        BigInt(amountE8s)
      );

      // Confirm the payment (in a real app, this would be done after actual ICP transfer)
      await fundFlowActor.confirm_payment(contributionId, backendActor.getCanisterPrincipal());

      reset();
      onOpenChange(false);
      onContributionSuccess();
    } catch (error) {
      console.error('Failed to contribute:', error);
      // You could add a toast notification here
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[400px]">
        <DialogHeader>
          <DialogTitle>Contribute ICP</DialogTitle>
          <DialogDescription>
            Make a contribution to "{campaignTitle}" using ICP coins.
          </DialogDescription>
        </DialogHeader>
        
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="amount">Amount (ICP)</Label>
            <Input
              id="amount"
              type="number"
              step="0.00000001"
              {...register('amount')}
              placeholder="0.00"
            />
            {errors.amount && (
              <p className="text-sm text-red-500">{errors.amount.message}</p>
            )}
            {amount && (
              <p className="text-sm text-muted-foreground">
                {formatCurrency(parseFloat(amount))} ({amountE8s.toLocaleString()} e8s)
              </p>
            )}
          </div>

          <div className="bg-muted p-3 rounded-lg">
            <h4 className="font-medium text-sm mb-2">Contribution Details</h4>
            <div className="space-y-1 text-sm text-muted-foreground">
              <div className="flex justify-between">
                <span>Campaign ID:</span>
                <span className="font-mono">{campaignId.toString()}</span>
              </div>
              <div className="flex justify-between">
                <span>Payment Method:</span>
                <span>ICP Transfer</span>
              </div>
              <div className="flex justify-between">
                <span>Network:</span>
                <span>Internet Computer</span>
              </div>
            </div>
          </div>

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={isLoading}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={isLoading}>
              {isLoading ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Processing...
                </>
              ) : (
                <>
                  <TrendingUp className="mr-2 h-4 w-4" />
                  Contribute
                </>
              )}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
};
