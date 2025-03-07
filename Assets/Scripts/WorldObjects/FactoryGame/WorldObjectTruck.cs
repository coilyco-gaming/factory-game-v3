using System;
using Assets.Scripts.Components.Core;
using Assets.Scripts.ScriptableObject;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    public class WorldObjectTruck : WorldObject
    {
        public uint totalVolumeCapacity = 500;
        public uint totalWeightCapacity = 500;
        public uint totalBatteryCapacity = 1000;

        public override void Instantiate(SpawnQueueItem spawnQueueItem)
        {
            base.Instantiate(spawnQueueItem);
            this.core.resources = new(
                new FactoryGameContent(),
                weightCapacity: this.totalWeightCapacity,
                volumeCapacity: this.totalVolumeCapacity
            );
            this.core.battery = new(capacity: this.totalBatteryCapacity);
            this.core.receiver = new(
                this.core,
                FactoryGameContent.DispatchVerbs.Deploy.ToString(), // Deploy
                FactoryGameContent.Spawnables.MiningDrill.ToString() // Mining drill
            );
        }

        protected override Func<StatusDataComponentCore.StatusData> GetStatusData()
        {
            return () =>
            {
                StatusDataComponentCore.StatusData statusData = new()
                {
                    Name = this.WorldObjectType,
                    Info = this.core.resources.ResourceInfo,
                };
                statusData.Info["Storage Volume"] = this.core.resources.UsedVolumeString;
                statusData.Info["Energy"] = this.core.battery.PercentEnergyStatus;
                statusData.Info["Target Location"] = this.core.receiver.targetPosition.ToString();
                return statusData;
            };
        }
    }
}
