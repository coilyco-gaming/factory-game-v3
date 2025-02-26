using System;
using System.Collections.Generic;
using Assets.Scripts.WorldObjects;
using Assets.Scripts.WorldObjects.FactoryGame;
using TMPro;
using UnityEngine;
using UnityEngine.UI;

// TODO: create a completed build that folks can "play" (run? play?)
namespace Assets.Scripts
{
    public class FactoryGameController : GameController
    {
        public GameObject resetButton;
        public GameObject pauseButton;
        public int HQOreBuffer = 3; // TODO: world init settings screen for these values
        public int spawnAttempts = 5;
        public float oreSpawnFactor = 0.5f;
        public int OreQuantityBase = 2000;
        public int OreQuantityRange = 1000;
        private TextMeshProUGUI pauseTextComponent;

        public enum Ores
        {
            Iron,
            Coal,
            Copper,
        }

        public enum Spawnables
        {
            Radar,
            CoalPlant,
            Drill,
            Factory,
            Warehouse,
        }

        public override void Start()
        {
            base.Start();

            Button resetComponent = this.resetButton.GetComponent<Button>();
            resetComponent.onClick.AddListener(this.Reset);

            Button pauseComponent = this.pauseButton.GetComponent<Button>();
            pauseComponent.onClick.AddListener(this.TogglePausePlay);

            this.pauseTextComponent = this.pauseButton.GetComponentInChildren<TextMeshProUGUI>();
            this.RenderPausePlay(true); // start paused

            this.Reset();
        }

        protected override void Reset()
        {
            base.Reset();
            this.SpawnOres(Ores.Iron.ToString());
            this.SpawnOres(Ores.Copper.ToString());
            this.SpawnOres(Ores.Coal.ToString());

            // Spawn some initial buildings
            //
            // C = Coal Plant
            // F = Factory
            // W = Warehouse
            // S = Radar
            //
            //   W
            // R C R
            // R C F
            //   W W
            //
            // Both coal plants have a warehouse beside them with a stockpile of coal.
            // The factory has a warehouse beside it with a stockpile of iron and copper.
            // There's 1 scanner for each resource: iron, copper, coal.

            System.Numerics.Vector2 HQPosition = new(
                this.Map.mapSize.x / 2,
                this.Map.mapSize.y / 2
            );

            this.Spawn(
                new SpawnQueueItem(
                    Spawnables.Radar.ToString(),
                    new System.Numerics.Vector2(HQPosition.X, HQPosition.Y), // for visual consistency
                    postInstantiateCallback: this.SpawnRadarCallback(Ores.Copper.ToString())
                )
            );

            this.Spawn(
                new SpawnQueueItem(
                    Spawnables.Radar.ToString(),
                    new System.Numerics.Vector2(HQPosition.X, HQPosition.Y + 1),
                    postInstantiateCallback: this.SpawnRadarCallback(Ores.Copper.ToString())
                )
            );

            this.Spawn(
                new SpawnQueueItem(
                    Spawnables.Radar.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 2, HQPosition.Y + 1),
                    postInstantiateCallback: this.SpawnRadarCallback(Ores.Iron.ToString())
                )
            );

            this.Spawn(
                new SpawnQueueItem(
                    Spawnables.CoalPlant.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 1, HQPosition.Y)
                )
            );

            this.Spawn(
                new SpawnQueueItem(
                    Spawnables.CoalPlant.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 1, HQPosition.Y + 1)
                )
            );

            this.Spawn(
                new SpawnQueueItem(
                    Spawnables.Factory.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 2, HQPosition.Y)
                )
            );

            this.Spawn(
                new SpawnQueueItem(
                    Spawnables.Warehouse.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 1, HQPosition.Y - 1),
                    postInstantiateCallback: this.SpawnCoalWarehouseCallback()
                )
            );

            this.Spawn(
                new SpawnQueueItem(
                    Spawnables.Warehouse.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 1, HQPosition.Y + 2),
                    postInstantiateCallback: this.SpawnCoalWarehouseCallback()
                )
            );

            this.Spawn(
                new SpawnQueueItem(
                    Spawnables.Warehouse.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 2, HQPosition.Y - 1),
                    postInstantiateCallback: this.SpawnFactoryWarehouseCallback()
                )
            );

            this.readyForTicks = false;
        }

        private void SpawnOres(string objectName)
        {
            int oresToSpawn = (int)(
                Mathf.Sqrt(
                    (float)(Math.Pow(this.Map.mapSize.x, 2) + Math.Pow(this.Map.mapSize.y, 2))
                ) * this.oreSpawnFactor
            );
            for (int i = 0; i < oresToSpawn; i++)
            {
                for (int attempts = 0; attempts < this.spawnAttempts; attempts++)
                {
                    // Start by picking a random position
                    int x = this.random.Next(0, this.Map.mapSize.x);
                    int y = this.random.Next(0, this.Map.mapSize.y);

                    // Draw a triangle to ensure that our ore is at least 2 blocks away
                    double distance = Math.Sqrt(Math.Pow(x, 2) + Math.Pow(y, 2));
                    if (distance <= this.HQOreBuffer)
                    {
                        continue;
                    }
                    System.Numerics.Vector2 position = new(x, y);

                    // Only spawn if there isn't already in ore at this position
                    bool oreAtPosition =
                        this.GetWorldObjectsByPositionAndType(
                            position,
                            new List<string>
                            {
                                Ores.Iron.ToString(),
                                Ores.Copper.ToString(),
                                Ores.Coal.ToString(),
                            }
                        ) != null;

                    // Spawn Ore
                    if (!oreAtPosition)
                    {
                        this.Spawn(
                            new SpawnQueueItem(
                                objectName,
                                position,
                                instantiateCallback: this.SpawnOreCallback()
                            )
                        );
                        break;
                    }
                }
            }
        }

        private Action<GameController, WorldObject> SpawnRadarCallback(string target)
        {
            return (gameController, worldObject) =>
            {
                WorldObjectRadar worldObjectRadar = worldObject as WorldObjectRadar;
                worldObjectRadar.Target = target;
            };
        }

        private Action<GameController, WorldObject> SpawnOreCallback()
        {
            return (gameController, worldObject) =>
            {
                float randomPercent = (float)this.random.Next(-100, 100) / 100;
                int oreQuantityChange = (int)(randomPercent * this.OreQuantityRange);
                uint oreQuantity = (uint)(this.OreQuantityBase + oreQuantityChange);
                WorldObjectOre worldObjectOre = worldObject as WorldObjectOre;
                worldObjectOre.Amount = oreQuantity;
            };
        }

        private Action<GameController, WorldObject> SpawnCoalWarehouseCallback()
        {
            return (gameController, worldObject) =>
            {
                WorldObjectWarehouse worldObjectWarehouse = worldObject as WorldObjectWarehouse;
                worldObjectWarehouse.Resources.CreateResources(Ores.Coal.ToString(), 1000);
            };
        }

        private Action<GameController, WorldObject> SpawnFactoryWarehouseCallback()
        {
            return (gameController, worldObject) =>
            {
                WorldObjectWarehouse worldObjectWarehouse = worldObject as WorldObjectWarehouse;
                worldObjectWarehouse.Resources.CreateResources(Ores.Iron.ToString(), 750);
                worldObjectWarehouse.Resources.CreateResources(Ores.Copper.ToString(), 250);
            };
        }

        private void TogglePausePlay()
        {
            this.readyForTicks = !this.readyForTicks; // toggle
            this.RenderPausePlay(!this.readyForTicks); // if not ready, then render paused
        }

        private void RenderPausePlay(bool paused)
        {
            string playText = "Play";
            string pauseText = "Pause";
            this.pauseTextComponent.SetText(paused ? pauseText : playText);
            this.PlayerComponent.ToggleFogPosition(paused); // if paused, then move flow closer
        }
    }
}
