import React from 'react';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '../components/ui/card';
import { Button } from '../components/ui/button';
import { Progress } from '../components/ui/progress';
import { Badge } from '../components/ui/badge';
import { formatE8s, formatCurrency, calculateProgress, getDaysLeft, truncateAddress } from '../lib/utils';
import { FUND_FLOW_CANISTER_ID_STR } from '../lib/ic';
import { Copy, ExternalLink, TrendingUp } from 'lucide-react';

interface CampaignCardProps {
  campaign: {
    id: bigint;
    title: string;
    goal: bigint;
    amount_raised: bigint;
    end_date: bigint;
    days_left: bigint;
    category: string;
    idea_id: bigint;
  };
  fundFlowActor: any;
  backendActor: any;
  onContribute: (campaignId: bigint) => void;
}

export const CampaignCard: React.FC<CampaignCardProps> = ({
  campaign,
  fundFlowActor,
  backendActor,
  onContribute,
}) => {
  const progress = calculateProgress(campaign.amount_raised, campaign.goal);
  const daysLeft = getDaysLeft(campaign.end_date);
  const isActive = daysLeft > 0;
  const isFunded = campaign.amount_raised >= campaign.goal;

  const copyAddress = async () => {
    try {
      await navigator.clipboard.writeText(FUND_FLOW_CANISTER_ID_STR);
    } catch (error) {
      console.error('Failed to copy address:', error);
    }
  };

  const getStatusBadge = () => {
    if (!isActive) {
      return <Badge variant="destructive">Ended</Badge>;
    }
    if (isFunded) {
      return <Badge variant="default" className="bg-green-500">Funded</Badge>;
    }
    return <Badge variant="secondary">Active</Badge>;
  };

  return (
    <Card className="w-full max-w-md hover:shadow-lg transition-shadow duration-200">
      <CardHeader>
        <div className="flex items-start justify-between">
          <div className="flex-1">
            <CardTitle className="text-lg font-semibold line-clamp-2">
              {campaign.title}
            </CardTitle>
            <CardDescription className="mt-2">
              <Badge variant="outline" className="mr-2">
                {campaign.category}
              </Badge>
              {getStatusBadge()}
            </CardDescription>
          </div>
        </div>
      </CardHeader>

      <CardContent className="space-y-4">
        {/* Funding Progress */}
        <div className="space-y-2">
          <div className="flex justify-between text-sm">
            <span className="text-muted-foreground">Progress</span>
            <span className="font-medium">{progress.toFixed(1)}%</span>
          </div>
          <Progress value={progress} className="h-2" />
          <div className="flex justify-between text-sm">
            <span className="text-muted-foreground">
              {formatCurrency(Number(campaign.amount_raised) / 100_000_000)} raised
            </span>
            <span className="text-muted-foreground">
              Goal: {formatCurrency(Number(campaign.goal) / 100_000_000)}
            </span>
          </div>
        </div>

        {/* Campaign Info */}
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div>
            <span className="text-muted-foreground">Campaign ID:</span>
            <p className="font-mono text-xs">{campaign.id.toString()}</p>
          </div>
          <div>
            <span className="text-muted-foreground">Days Left:</span>
            <p className={daysLeft > 0 ? 'text-green-600' : 'text-red-600'}>
              {daysLeft > 0 ? `${daysLeft} days` : 'Ended'}
            </p>
          </div>
        </div>

        {/* Deposit Address */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-sm text-muted-foreground">Deposit Address</span>
            <Button
              variant="ghost"
              size="sm"
              onClick={copyAddress}
              className="h-6 w-6 p-0"
            >
              <Copy className="h-3 w-3" />
            </Button>
          </div>
          <div className="bg-muted p-2 rounded text-xs font-mono break-all">
            {truncateAddress(FUND_FLOW_CANISTER_ID_STR)}
          </div>
        </div>
      </CardContent>

      <CardFooter className="flex gap-2">
        <Button
          onClick={() => onContribute(campaign.id)}
          disabled={!isActive}
          className="flex-1"
        >
          <TrendingUp className="h-4 w-4 mr-2" />
          Contribute ICP
        </Button>
        <Button variant="outline" size="sm">
          <ExternalLink className="h-4 w-4" />
        </Button>
      </CardFooter>
    </Card>
  );
};
