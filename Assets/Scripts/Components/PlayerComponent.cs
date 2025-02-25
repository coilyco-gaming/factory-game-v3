#if UNITY_6000
namespace Assets.Scripts.Components.Unity
{
    using UnityEngine;

    public class PlayerComponent : MonoBehaviour
    {
        public int maxZoom = 10;
        public GameObject backgroundFog;
        private int xSize;
        private int ySize;

        public void Instantiate(int xSize, int ySize)
        {
            this.xSize = xSize;
            this.ySize = ySize;
            this.Reset();
        }

        public void Reset()
        {
            Camera.main.transform.position = new Vector3(
                this.xSize / 2,
                this.ySize / 2,
                Camera.main.transform.position.z
            );
        }

        public void Update()
        {
            if (this.GetKey(KeyCode.W))
            {
                Camera.main.transform.position += Vector3.up;
            }
            if (this.GetKey(KeyCode.A))
            {
                Camera.main.transform.position += Vector3.left;
            }
            if (this.GetKey(KeyCode.S))
            {
                Camera.main.transform.position += Vector3.down;
            }
            if (this.GetKey(KeyCode.D))
            {
                Camera.main.transform.position += Vector3.right;
            }
            if (Input.mouseScrollDelta.y > 0 && Camera.main.orthographicSize > 1)
            {
                Camera.main.orthographicSize--;
            }
            if (Input.mouseScrollDelta.y < 0 && Camera.main.orthographicSize < this.maxZoom)
            {
                Camera.main.orthographicSize++;
            }
        }

        public System.Numerics.Vector2 GetGridPosition()
        {
            return new System.Numerics.Vector2(
                Camera.main.transform.position.x,
                Camera.main.transform.position.y
            );
        }

        public void ToggleFogPosition(bool close)
        {
            this.backgroundFog.transform.position = close
                ? new Vector3(0, 0, 50)
                : new Vector3(0, 0, 300);
        }

        // Allows for "slow" movement when pressing the key,
        // and "fast" movement when holding shift.
        private bool GetKey(KeyCode keyCode)
        {
            return Input.GetKeyDown(keyCode)
                || (
                    (Input.GetKey(KeyCode.LeftShift) || Input.GetKey(KeyCode.RightShift))
                    && Input.GetKey(keyCode)
                );
        }
    }
}
#endif
