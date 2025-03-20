using System;
using System.Collections.Generic;
using System.Linq;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    [Serializable]
    public class WorldObjectMiningDrill : WorldObject
    {
        private uint totalVolumeCapacity = 1000;
        private uint totalBatteryCapacity = 1000;
        private int miningSpeed = 5;
        private int miningEnergyCost = 5;
        public override float ZIndex => 2; // TODO: make this a constant

        public override StatusDataComponentCore StatusData =>
            new()
            {
                Name = Util.HumanizedString(this.WorldObjectType),
                Energy = this.core.battery.PercentEnergyStatus,
                Dispatchers = this
                    .core.dispatchers.Select(dispatcher => dispatcher.Description)
                    .ToList(),
                Resources = this.core.resources.ResourceInfo,
                Info = new()
                {
                    { "Outputs", Util.HumanizedString(this.core.targetType).ToLower() },
                    { "Storage Volume", this.core.resources.UsedVolumeString },
                },
                Alerts = this.core.alerts.Count == 0 ? null : this.core.alerts,
            };

        public override void Instantiate(SpawnQueueItem spawnQueueItem, GameContent gameContent)
        {
            base.Instantiate(spawnQueueItem, gameContent);
            this.core.passThrough = true;
            this.core.resources = new(
                gameContent,
                weightCapacity: uint.MaxValue,
                volumeCapacity: this.totalVolumeCapacity
            )
            {
                resources = spawnQueueItem.resources,
            };
            this.core.battery = new(capacity: this.totalBatteryCapacity);
            this.core.dispatchers = new List<DispatchComponentCore>
            {
                // TODO: adjacent stone mining drill if necessary
                new(
                    gameContent,
                    this.core,
                    // Collect...
                    DispatchComponentCore.Verbs.Collect.ToString(),
                    // ...< product >...
                    this.core.targetType,
                    // ...from me.
                    DispatchComponentCore.Keywords.Me.ToString()
                ),
            };
            this.core.mining = new MiningComponentCore(
                gameContent,
                this.core.targetType,
                this.miningSpeed,
                this.miningEnergyCost
            );
            this.core.powerLine = new PowerLineComponentCore(
                FactoryGameContent.Spawnables.PowerLines.ToString()
            );
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            this.core.CreateAlert(
                gameController.core,
                this.core.battery.Tick(gameController.core, this.core)
            );
            foreach (DispatchComponentCore dispatcher in this.core.dispatchers)
            {
                this.core.CreateAlert(
                    gameController.core,
                    dispatcher.Tick(gameController.core, this.core)
                );
            }
            this.core.CreateAlert(
                gameController.core,
                this.core.mining.Tick(gameController.core, this.core)
            );
            this.core.CreateAlert(
                gameController.core,
                this.core.powerLine.Tick(gameController.core, this.core)
            );

            // If no ore world object is on our position
            bool oreAtPosition =
                gameController
                    .core.worldObjects.GetValueOrDefault(this.GridPosition)
                    ?.Any(worldObject => worldObject.Value.backref is WorldObjectOre ore) ?? false;

            // If our resources are empty
            bool resourcesEmpty = !this.core.resources.HasResources;

            // Then delete the mining drill
            if (!oreAtPosition && resourcesEmpty)
            {
                gameController.QueueForDeletion(
                    new DeletionQueueItem(this.core, this.GridPosition)
                );
            }
        }
    }
}
