import React, { useState, useEffect } from 'react';
import { Button } from './components/ui/button';
import { CampaignCard } from './components/CampaignCard';
import { CreateProjectDialog } from './components/CreateProjectDialog';
import { ContributionDialog } from './components/ContributionDialog';
import { Dashboard } from './components/Dashboard';
import { createFundVerseBackendActor, createFundFlowActor, login, isAuthenticated } from './lib/ic';
import { Wallet, LogOut, BarChart3, Grid } from 'lucide-react';

function App() {
  const [backendActor, setBackendActor] = useState<any>(null);
  const [fundFlowActor, setFundFlowActor] = useState<any>(null);
  const [campaigns, setCampaigns] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [authenticated, setAuthenticated] = useState(false);
  const [view, setView] = useState<'grid' | 'dashboard'>('grid');
  const [contributionDialog, setContributionDialog] = useState<{
    open: boolean;
    campaignId: bigint;
    campaignTitle: string;
  }>({
    open: false,
    campaignId: BigInt(0),
    campaignTitle: '',
  });

  // Initialize actors and authentication
  useEffect(() => {
    const initializeApp = async () => {
      try {
        const authStatus = await isAuthenticated();
        setAuthenticated(authStatus);

        if (authStatus) {
          const backend = await createFundVerseBackendActor();
          const fundFlow = await createFundFlowActor();
          setBackendActor(backend);
          setFundFlowActor(fundFlow);
        }
      } catch (error) {
        console.error('Failed to initialize app:', error);
      } finally {
        setLoading(false);
      }
    };

    initializeApp();
  }, []);

  // Load campaigns when actors are available
  useEffect(() => {
    if (backendActor) {
      loadCampaigns();
    }
  }, [backendActor]);

  const loadCampaigns = async () => {
    if (!backendActor) return;

    try {
      const campaignCards = await backendActor.get_campaign_cards();
      setCampaigns(campaignCards);
    } catch (error) {
      console.error('Failed to load campaigns:', error);
    }
  };

  const handleLogin = async () => {
    try {
      await login();
      setAuthenticated(true);
      
      // Re-initialize actors after login
      const backend = await createFundVerseBackendActor();
      const fundFlow = await createFundFlowActor();
      setBackendActor(backend);
      setFundFlowActor(fundFlow);
    } catch (error) {
      console.error('Login failed:', error);
    }
  };

  const handleLogout = async () => {
    try {
      // You would implement logout logic here
      setAuthenticated(false);
      setBackendActor(null);
      setFundFlowActor(null);
      setCampaigns([]);
    } catch (error) {
      console.error('Logout failed:', error);
    }
  };

  const handleContribute = (campaignId: bigint) => {
    const campaign = campaigns.find(c => c.id === campaignId);
    if (campaign) {
      setContributionDialog({
        open: true,
        campaignId,
        campaignTitle: campaign.title,
      });
    }
  };

  const handleContributionSuccess = () => {
    loadCampaigns(); // Refresh campaigns after contribution
  };

  const handleProjectCreated = () => {
    loadCampaigns(); // Refresh campaigns after project creation
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-primary mx-auto"></div>
          <p className="mt-4 text-lg">Loading FundVerse...</p>
        </div>
      </div>
    );
  }

  if (!authenticated) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100">
        <div className="text-center space-y-6">
          <div className="space-y-2">
            <h1 className="text-4xl font-bold text-gray-900">FundVerse</h1>
            <p className="text-xl text-gray-600">Decentralized Funding Platform</p>
          </div>
          <div className="space-y-4">
            <p className="text-gray-500 max-w-md">
              Connect your Internet Computer wallet to start funding projects with ICP coins
            </p>
            <Button onClick={handleLogin} size="lg" className="flex items-center gap-2">
              <Wallet className="h-5 w-5" />
              Connect Wallet
            </Button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b bg-card">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              <h1 className="text-2xl font-bold">FundVerse</h1>
              <div className="flex items-center space-x-2">
                <Button
                  variant={view === 'grid' ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => setView('grid')}
                >
                  <Grid className="h-4 w-4" />
                </Button>
                <Button
                  variant={view === 'dashboard' ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => setView('dashboard')}
                >
                  <BarChart3 className="h-4 w-4" />
                </Button>
              </div>
            </div>
            
            <div className="flex items-center space-x-4">
              {backendActor && (
                <CreateProjectDialog
                  backendActor={backendActor}
                  onProjectCreated={handleProjectCreated}
                />
              )}
              <Button variant="outline" onClick={handleLogout} className="flex items-center gap-2">
                <LogOut className="h-4 w-4" />
                Disconnect
              </Button>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="container mx-auto px-4 py-8">
        {view === 'dashboard' ? (
          <Dashboard campaigns={campaigns} />
        ) : (
          <div className="space-y-6">
            {/* Campaigns Grid */}
            <div className="flex items-center justify-between">
              <div>
                <h2 className="text-3xl font-bold">Campaigns</h2>
                <p className="text-muted-foreground">
                  Discover and fund innovative projects with ICP
                </p>
              </div>
              <div className="text-right">
                <p className="text-2xl font-bold">{campaigns.length}</p>
                <p className="text-sm text-muted-foreground">Total Campaigns</p>
              </div>
            </div>

            {campaigns.length === 0 ? (
              <div className="text-center py-12">
                <div className="mx-auto w-24 h-24 bg-muted rounded-full flex items-center justify-center mb-4">
                  <BarChart3 className="h-12 w-12 text-muted-foreground" />
                </div>
                <h3 className="text-lg font-semibold mb-2">No campaigns yet</h3>
                <p className="text-muted-foreground mb-4">
                  Be the first to create a funding campaign!
                </p>
                {backendActor && (
                  <CreateProjectDialog
                    backendActor={backendActor}
                    onProjectCreated={handleProjectCreated}
                  />
                )}
              </div>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {campaigns.map((campaign) => (
                  <CampaignCard
                    key={campaign.id.toString()}
                    campaign={campaign}
                    fundFlowActor={fundFlowActor!}
                    backendActor={backendActor!}
                    onContribute={handleContribute}
                  />
                ))}
              </div>
            )}
          </div>
        )}
      </main>

      {/* Contribution Dialog */}
      {contributionDialog.open && fundFlowActor && backendActor && (
        <ContributionDialog
          open={contributionDialog.open}
          onOpenChange={(open) => setContributionDialog(prev => ({ ...prev, open }))}
          campaignId={contributionDialog.campaignId}
          campaignTitle={contributionDialog.campaignTitle}
          fundFlowActor={fundFlowActor}
          backendActor={backendActor}
          onContributionSuccess={handleContributionSuccess}
        />
      )}
    </div>
  );
}

export default App;
