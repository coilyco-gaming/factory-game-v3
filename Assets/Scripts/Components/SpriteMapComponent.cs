#if UNITY_6000
namespace Assets.Scripts.Components.Unity
{
    using System.Collections.Generic;
    using Assets.Scripts.WorldObjects;
    using Roy_T.AStar.Graphs;
    using Roy_T.AStar.Paths;
    using Roy_T.AStar.Primitives;
    using UnityEngine;

    public class SpriteMapComponent : MonoBehaviour
    {
        public UnityEngine.Vector2Int mapSize = new(20, 20);
        public GameObject WorldGameObject { get; set; }

        private Canvas parentCanvas;
        private Dictionary<Vector2, GameObject> tiles = new();

        private GameObject MapGameObject { get; set; }
        private GameObject TilesGameObject { get; set; }

        public void Instantiate(Canvas parentCanvas)
        {
            this.parentCanvas = parentCanvas;
            this.MapGameObject = new("Map");
            this.MapGameObject.transform.SetParent(this.transform);
            this.TilesGameObject = new("Tiles");
            this.TilesGameObject.transform.SetParent(this.MapGameObject.transform);
            this.WorldGameObject = new("World");
            this.WorldGameObject.transform.SetParent(this.MapGameObject.transform);

            float childZPosition = this.parentCanvas.planeDistance;
            UnityEngine.Vector3 childPosition = new(0, 0, childZPosition);
            this.MapGameObject.transform.position = childPosition;

            for (int x = 0; x < this.mapSize.x; x++)
            {
                for (int y = 0; y < this.mapSize.y; y++)
                {
                    Vector2 position = new(x, y);
                    GameObject tile = this.GenerateTile(position);
                    this.tiles.Add(position, tile);
                }
            }
        }

        private GameObject GenerateTile(Vector2 position)
        {
            GameObject tile = new($"Tile ({position.x}, {position.y})");
            tile.transform.SetParent(this.TilesGameObject.transform);
            tile.transform.localPosition = new Vector3(position.x, position.y, 0);
            tile.transform.localScale = new Vector3(1, 1, 1);

            SpriteRenderer sprite = tile.AddComponent<SpriteRenderer>();
            sprite.sprite = Resources.Load<Sprite>("Art/dirt");

            return tile;
        }

        public System.Numerics.Vector2 DiamondSpiralPattern(
            System.Numerics.Vector2 origin,
            System.Numerics.Vector2 currentTarget
        )
        {
            System.Numerics.Vector2 changeVector = currentTarget - origin;

            if (changeVector.X == 0 && changeVector.Y == 0)
            {
                currentTarget.Y += 1;
                return currentTarget;
            }

            if (changeVector.X >= 0 && changeVector.Y > 0)
            {
                currentTarget.X += 1;
                currentTarget.Y -= 1;
                return currentTarget;
            }

            if (changeVector.X > 0 && changeVector.Y <= 0)
            {
                currentTarget.X -= 1;
                currentTarget.Y -= 1;
                return currentTarget;
            }

            if (changeVector.X <= 0 && changeVector.Y < 0)
            {
                currentTarget.X -= 1;
                currentTarget.Y += 1;
                return currentTarget;
            }

            if (changeVector.X < 0 && changeVector.Y >= 0)
            {
                currentTarget.X += 1;
                currentTarget.Y += 1;
                // If the change vector would return you to 0,N
                // then add +1 to the Y
                if (currentTarget.X == origin.X)
                {
                    currentTarget.Y += 1;
                }
                return currentTarget;
            }

            return currentTarget;
        }

        public System.Numerics.Vector2? GetMovement(
            GameController gameComponent,
            System.Numerics.Vector2 start,
            System.Numerics.Vector2 end
        )
        {
            System.Numerics.Vector2 movement;

            // Setup plain grid
            GridSize gridSize = new(this.mapSize.x, this.mapSize.y);
            Roy_T.AStar.Grids.Grid grid =
                Roy_T.AStar.Grids.Grid.CreateGridWithLateralAndDiagonalConnections(
                    gridSize,
                    new Size(Distance.FromMeters(1), Distance.FromMeters(1)),
                    Velocity.FromMetersPerSecond(1)
                );

            // Register obstacles
            foreach (
                KeyValuePair<
                    System.Numerics.Vector2,
                    Dictionary<string, WorldObject>
                > worldObjects in gameComponent.GetWorldObjects()
            )
            {
                System.Numerics.Vector2 position = worldObjects.Key;
                // Nothing here
                if (worldObjects.Value == null || worldObjects.Value.Count == 0)
                {
                    continue;
                }
                // Dont block on yourself
                if (position.X == start.X && position.Y == start.Y)
                {
                    continue;
                }
                // Dont block on the target
                if (position.X == end.X && position.Y == end.Y)
                {
                    continue;
                }
                // Register obstacles
                grid.DisconnectNode(new GridPosition((int)position.X, (int)position.Y));
            }

            // Find the path
            PathFinder pathFinder = new();
            Path path = pathFinder.FindPath(
                new GridPosition((int)start.X, (int)start.Y),
                new GridPosition((int)end.X, (int)end.Y),
                grid
            );

            // Derive the movement vector from the next node on the path
            // TODO: what if you can't find a path? wait? self destruct?
            if (path == null || path.Edges.Count == 0)
            {
                return null;
            }

            IEdge edge = path.Edges[0];
            System.Numerics.Vector2 nextPosition = new(edge.End.Position.X, edge.End.Position.Y);
            movement = nextPosition - start;

            return movement;
        }
    }
}
#endif
