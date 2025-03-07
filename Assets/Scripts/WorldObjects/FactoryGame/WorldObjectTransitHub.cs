using System;
using Assets.Scripts.Components.Core;
using Assets.Scripts.ScriptableObject;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    [Serializable]
    public class WorldObjectTransitHub : WorldObject
    {
        public uint totalBatteryCapacity = 10000;

        public override void Instantiate(SpawnQueueItem spawnQueueItem)
        {
            base.Instantiate(spawnQueueItem);
            this.core.battery = new(capacity: this.totalBatteryCapacity);
            this.core.hub = new(new FactoryGameContent(), this.core, this.core.battery);
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            this.core.hub.Balance(gameController.core);
        }

        protected override Func<StatusDataComponentCore.StatusData> GetStatusData()
        {
            return () =>
            {
                StatusDataComponentCore.StatusData statusData = new()
                {
                    Name = this.WorldObjectType,
                    Info = new() { ["Energy"] = this.core.battery.PercentEnergyStatus },
                };
                return statusData;
            };
        }
    }
}
